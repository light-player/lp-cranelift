use crate::error::{ErrorCode, GlslError, extract_span_from_expr, source_span_to_location};
use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::codegen::lvalue::emit_lvalue_as_rvalue;
use crate::frontend::codegen::rvalue::RValue;
use crate::semantic::types::Type as GlslType;
use cranelift_codegen::ir::{InstBuilder, TrapCode, Value, condcodes::IntCC, types};
use glsl::syntax::{Expr, SourceSpan};

use alloc::vec::Vec;
use hashbrown::HashSet;

/// Component naming sets for vector swizzles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamingSet {
    XYZW, // Position/generic: x, y, z, w
    RGBA, // Color: r, g, b, a
    STPQ, // Texture coordinates: s, t, p, q
}

pub fn translate_component_access(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    // Ensure we're in a block before evaluating
    ctx.ensure_block()?;

    let Expr::Dot(base_expr, field, dot_span) = expr else {
        unreachable!("translate_component_access called on non-dot expr");
    };

    let (vals, ty) = ctx.translate_expr_typed(base_expr)?;

    if !ty.is_vector() {
        let span = extract_span_from_expr(base_expr);
        let error = GlslError::new(
            ErrorCode::E0112,
            format!("component access on non-vector type: {:?}", ty),
        )
        .with_location(source_span_to_location(&span));
        return Err(ctx.add_span_to_error(error, &span));
    }

    // Use the span from the dot expression for error reporting
    let indices = parse_vector_swizzle(&field.name, &ty, Some(dot_span.clone()))?;
    let base_ty = ty.vector_base_type().unwrap();

    if indices.len() == 1 {
        // Single component: return scalar
        Ok((vec![vals[indices[0]]], base_ty))
    } else {
        // Multi-component: return vector
        let result_vals: Vec<Value> = indices.iter().map(|&idx| vals[idx]).collect();
        let result_ty = GlslType::vector_type(&base_ty, indices.len()).ok_or_else(|| {
            GlslError::new(
                ErrorCode::E0400,
                format!("cannot create vector of size {}", indices.len()),
            )
        })?;
        Ok((result_vals, result_ty))
    }
}

pub fn translate_matrix_indexing(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    // Ensure we're in a block before evaluating
    ctx.ensure_block()?;

    let Expr::Bracket(array_expr, array_spec, span) = expr else {
        unreachable!("translate_matrix_indexing called on non-bracket expr");
    };

    let (array_vals, array_ty) = ctx.translate_expr_typed(array_expr)?;
    crate::debug!(
        "translate_matrix_indexing: array_ty={:?}, array_vals.len()={}",
        array_ty,
        array_vals.len()
    );

    if !array_ty.is_matrix() && !array_ty.is_vector() {
        return Err(GlslError::new(
            ErrorCode::E0400,
            "indexing only supported for matrices and vectors",
        )
        .with_location(source_span_to_location(span)));
    }

    // Extract index expressions from ArraySpecifier
    // ArraySpecifier can have multiple dimensions: mat[0][0] is parsed as one bracket with two dimensions
    use glsl::syntax::ArraySpecifierDimension;
    if array_spec.dimensions.0.is_empty() {
        return Err(
            GlslError::new(ErrorCode::E0400, "indexing requires explicit index")
                .with_location(source_span_to_location(span)),
        );
    }

    // Process dimensions one at a time
    let mut current_vals = array_vals;
    let mut current_ty = array_ty;

    for (_dim_idx, dimension) in array_spec.dimensions.0.iter().enumerate() {
        let index_expr = match dimension {
            ArraySpecifierDimension::ExplicitlySized(expr) => expr,
            ArraySpecifierDimension::Unsized => {
                return Err(
                    GlslError::new(ErrorCode::E0400, "indexing requires explicit index")
                        .with_location(source_span_to_location(span)),
                );
            }
        };

        // Evaluate index (must be int)
        let (index_vals, index_ty) = ctx.translate_expr_typed(index_expr)?;
        if index_ty != GlslType::Int {
            return Err(GlslError::new(ErrorCode::E0106, "index must be int")
                .with_location(source_span_to_location(span)));
        }

        // Check if index is compile-time constant or variable
        let index_val = index_vals[0];
        let is_constant = if let Expr::IntConst(n, _) = index_expr.as_ref() {
            Some(*n as usize)
        } else {
            None
        };

        if current_ty.is_matrix() {
            // Matrix indexing: mat[col] returns column vector
            let (rows, cols) = current_ty.matrix_dims().unwrap();
            let column_type = current_ty.matrix_column_type().unwrap();

            if let Some(index) = is_constant {
                // Compile-time constant index
                if index >= cols {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "matrix column index {} out of bounds (max {})",
                            index,
                            cols - 1
                        ),
                    )
                    .with_location(source_span_to_location(span)));
                }

                // Extract column elements
                // Matrix is stored column-major: [col0_row0, col0_row1, ..., col1_row0, ...]
                let mut column_vals = Vec::new();
                for row in 0..rows {
                    let idx = index * rows + row;
                    column_vals.push(current_vals[idx]);
                }

                current_vals = column_vals;
                current_ty = column_type;
            } else {
                // Variable index - need dynamic column extraction
                // Bounds check
                emit_bounds_check(ctx, index_val, cols, span)?;

                // Build select chain to choose column
                // For each possible column, extract all rows and build a select chain
                let mut column_candidates: Vec<Vec<Value>> = Vec::new();
                for col in 0..cols {
                    let mut column_vals = Vec::new();
                    for row in 0..rows {
                        let idx = col * rows + row;
                        column_vals.push(current_vals[idx]);
                    }
                    column_candidates.push(column_vals);
                }

                // Build select chain for each row component
                let mut result_column = Vec::new();
                for row_idx in 0..rows {
                    // Start with last column as default
                    let mut result = column_candidates[cols - 1][row_idx];
                    // Work backwards through columns
                    for col in (0..cols - 1).rev() {
                        let col_const = ctx.builder.ins().iconst(types::I32, col as i64);
                        let is_match = ctx.builder.ins().icmp(IntCC::Equal, index_val, col_const);
                        result = ctx.builder.ins().select(
                            is_match,
                            column_candidates[col][row_idx],
                            result,
                        );
                    }
                    result_column.push(result);
                }

                current_vals = result_column;
                current_ty = column_type;
            }
        } else if current_ty.is_vector() {
            // Vector indexing: vec[index] returns scalar component
            let component_count = current_ty.component_count().unwrap();

            if let Some(index) = is_constant {
                // Compile-time constant index
                crate::debug!(
                    "vector indexing: current_ty={:?}, index={}, component_count={}",
                    current_ty,
                    index,
                    component_count
                );

                if index >= component_count {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        format!(
                            "vector component index {} out of bounds (max {})",
                            index,
                            component_count - 1
                        ),
                    )
                    .with_location(source_span_to_location(span)));
                }

                let base_type = current_ty.vector_base_type().unwrap();
                crate::debug!(
                    "  extracted component: base_type={:?}, val index={}",
                    base_type,
                    index
                );
                current_vals = vec![current_vals[index]];
                current_ty = base_type;
                crate::debug!(
                    "  after extraction: current_ty={:?}, current_vals.len()={}",
                    current_ty,
                    current_vals.len()
                );
            } else {
                // Variable index - use dynamic selection
                // Bounds check
                emit_bounds_check(ctx, index_val, component_count, span)?;

                // Build select chain: result = (index == 0) ? vals[0] : ((index == 1) ? vals[1] : ...)
                let base_type = current_ty.vector_base_type().unwrap();
                let mut result = current_vals[component_count - 1]; // Default to last component
                for i in (0..component_count - 1).rev() {
                    let i_const = ctx.builder.ins().iconst(types::I32, i as i64);
                    let is_match = ctx.builder.ins().icmp(IntCC::Equal, index_val, i_const);
                    result = ctx.builder.ins().select(is_match, current_vals[i], result);
                }

                current_vals = vec![result];
                current_ty = base_type;
            }
        } else {
            return Err(GlslError::new(
                ErrorCode::E0400,
                format!(
                    "cannot index into {:?} (only matrices and vectors can be indexed)",
                    current_ty
                ),
            )
            .with_location(source_span_to_location(span)));
        }
    }

    crate::debug!(
        "translate_matrix_indexing result: current_ty={:?}, current_vals.len()={}",
        current_ty,
        current_vals.len()
    );
    Ok((current_vals, current_ty))
}

/// Parse vector component swizzle and return indices
/// Supports xyzw, rgba, stpq naming sets
/// Can parse multiple components: "xy", "rgba", "zyx", "xxxx", etc.
pub fn parse_vector_swizzle(
    name: &str,
    vec_ty: &GlslType,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<Vec<usize>, GlslError> {
    if name.is_empty() {
        return Err(GlslError::new(ErrorCode::E0113, "empty swizzle"));
    }

    if name.len() > 4 {
        return Err(GlslError::new(
            ErrorCode::E0113,
            format!("swizzle can have at most 4 components, got {}", name.len()),
        ));
    }

    let component_count = vec_ty.component_count().ok_or_else(|| {
        GlslError::new(
            ErrorCode::E0112,
            format!("{:?} is not a vector type", vec_ty),
        )
    })?;

    // Determine which naming set is used and validate consistency
    let naming_set = determine_naming_set(name)?;

    // Parse each character
    let mut indices = Vec::new();
    for ch in name.chars() {
        let idx = parse_single_component(ch, naming_set)?;

        // Validate index is within bounds
        if idx >= component_count {
            let mut error = GlslError::new(
                ErrorCode::E0111,
                format!(
                    "component '{}' not valid for {:?} (has only {} components)",
                    ch, vec_ty, component_count
                ),
            );
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            return Err(error);
        }

        indices.push(idx);
    }

    Ok(indices)
}

/// Determine which naming set is used in a swizzle and validate consistency
fn determine_naming_set(swizzle: &str) -> Result<NamingSet, GlslError> {
    let mut xyzw_count = 0;
    let mut rgba_count = 0;
    let mut stpq_count = 0;

    for ch in swizzle.chars() {
        match ch {
            'x' | 'y' | 'z' | 'w' => xyzw_count += 1,
            'r' | 'g' | 'b' | 'a' => rgba_count += 1,
            's' | 't' | 'p' | 'q' => stpq_count += 1,
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0113,
                    format!("invalid swizzle character: '{}'", ch),
                ));
            }
        }
    }

    let sets_used = (xyzw_count > 0) as u32 + (rgba_count > 0) as u32 + (stpq_count > 0) as u32;

    if sets_used > 1 {
        return Err(GlslError::new(
            ErrorCode::E0113,
            format!(
                "swizzle '{}' mixes component naming sets (xyzw/rgba/stpq)",
                swizzle
            ),
        ));
    }

    if xyzw_count > 0 {
        Ok(NamingSet::XYZW)
    } else if rgba_count > 0 {
        Ok(NamingSet::RGBA)
    } else {
        Ok(NamingSet::STPQ)
    }
}

/// Parse a single component character given a naming set
fn parse_single_component(ch: char, naming_set: NamingSet) -> Result<usize, GlslError> {
    match naming_set {
        NamingSet::XYZW => match ch {
            'x' => Ok(0),
            'y' => Ok(1),
            'z' => Ok(2),
            'w' => Ok(3),
            _ => Err(GlslError::new(
                ErrorCode::E0113,
                format!("invalid component '{}' for xyzw naming set", ch),
            )),
        },
        NamingSet::RGBA => match ch {
            'r' => Ok(0),
            'g' => Ok(1),
            'b' => Ok(2),
            'a' => Ok(3),
            _ => Err(GlslError::new(
                ErrorCode::E0113,
                format!("invalid component '{}' for rgba naming set", ch),
            )),
        },
        NamingSet::STPQ => match ch {
            's' => Ok(0),
            't' => Ok(1),
            'p' => Ok(2),
            'q' => Ok(3),
            _ => Err(GlslError::new(
                ErrorCode::E0113,
                format!("invalid component '{}' for stpq naming set", ch),
            )),
        },
    }
}

/// Check if a slice of indices contains duplicates
pub fn has_duplicates(indices: &[usize]) -> bool {
    let mut seen = HashSet::new();
    for &idx in indices {
        if !seen.insert(idx) {
            return true;
        }
    }
    false
}

/// Emit bounds checking code with trap for out-of-bounds indices
///
/// Checks that `index_val` is in range [0, bound) and traps if not.
/// Uses `TrapCode::user(1)` for "vector/matrix index out of bounds".
///
/// Uses `trapnz` to trap when the out-of-bounds condition is non-zero (true).
/// NOTE: The trap instruction is being generated correctly in the CLIF IR,
/// but it's not triggering at runtime in the emulator. This needs further
/// investigation - the trap instruction is emitted, but execution doesn't
/// seem to trigger it or the emulator isn't handling it correctly. The test
/// file `bvec2/index-variable-bounds.glsl` contains tests that expect traps
/// but currently fail because traps aren't being triggered.
///
/// TODO: Investigate why traps aren't triggering at runtime. Possible causes:
/// - The emulator might not be handling trap instructions correctly
/// - The trapnz lowering might not be implemented correctly for the target ISA
/// - There might be an issue with how trap instructions are executed
fn emit_bounds_check(
    ctx: &mut CodegenContext,
    index_val: Value,
    bound: usize,
    span: &SourceSpan,
) -> Result<(), GlslError> {
    // Ensure we're in a block
    ctx.ensure_block()?;

    // Set source location for trap instruction
    let srcloc = ctx.source_loc_manager().create_srcloc(span);
    ctx.builder.set_srcloc(srcloc);

    // Check: index < 0 || index >= bound
    let zero = ctx.builder.ins().iconst(types::I32, 0);
    let bound_val = ctx.builder.ins().iconst(types::I32, bound as i64);
    let index_lt_zero = ctx
        .builder
        .ins()
        .icmp(IntCC::SignedLessThan, index_val, zero);
    let index_ge_bound =
        ctx.builder
            .ins()
            .icmp(IntCC::SignedGreaterThanOrEqual, index_val, bound_val);
    let out_of_bounds = ctx.builder.ins().bor(index_lt_zero, index_ge_bound);

    // Use trapnz to trap when out_of_bounds is non-zero (true)
    // NOTE: trapnz may not be available in the lowering code yet, but we use it here
    // and will fix the lowering code later if needed.
    let trap_code = TrapCode::user(1).unwrap();
    ctx.builder.ins().trapnz(out_of_bounds, trap_code);

    Ok(())
}

/// Emit component access expression as RValue
///
/// Handles dot notation (e.g., `vec.x`, `vec.xy`) for both LValues and RValues.
/// For LValues (variables), resolves as LValue then loads.
/// For RValues (expressions), evaluates the expression then extracts components.
pub fn emit_component_access_rvalue(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<RValue, GlslError> {
    // Try LValue path first (for variables like `a.x`)
    // This is more efficient as it can directly access variable storage
    match ctx.emit_lvalue(expr) {
        Ok(lvalue) => ctx.load_lvalue(lvalue),
        Err(_) => {
            // Fall back to RValue path (for expressions like `not(bvec2(...)).x`)
            // This evaluates the expression first, then extracts components
            let (vals, ty) = translate_component_access(ctx, expr)?;
            Ok(RValue::from_aggregate(vals, ty))
        }
    }
}

/// Emit matrix/vector indexing expression as RValue
///
/// Handles bracket notation (e.g., `vec[0]`, `mat[0][1]`).
/// For constant indices, uses LValue path for efficiency.
/// For variable indices, uses translate_matrix_indexing directly.
pub fn emit_matrix_indexing_rvalue(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<RValue, GlslError> {
    // Check if this is a variable index - if so, use translate_matrix_indexing directly
    let Expr::Bracket(_, array_spec, _) = expr else {
        unreachable!("emit_matrix_indexing_rvalue called on non-bracket expr");
    };

    use glsl::syntax::ArraySpecifierDimension;
    let has_variable_index = array_spec.dimensions.0.iter().any(|dim| match dim {
        ArraySpecifierDimension::ExplicitlySized(index_expr) => {
            !matches!(index_expr.as_ref(), Expr::IntConst(_, _))
        }
        ArraySpecifierDimension::Unsized => false,
    });

    if has_variable_index {
        // Variable index - use translate_matrix_indexing directly
        let (vals, ty) = translate_matrix_indexing(ctx, expr)?;
        Ok(RValue::from_aggregate(vals, ty))
    } else {
        // Constant index - use LValue path for efficiency
        emit_lvalue_as_rvalue(ctx, expr)
    }
}
