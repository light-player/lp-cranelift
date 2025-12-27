use crate::error::{ErrorCode, GlslError, extract_span_from_expr, source_span_to_location};
use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::codegen::lvalue::emit_lvalue_as_rvalue;
use crate::frontend::codegen::rvalue::RValue;
use crate::semantic::types::Type as GlslType;
use cranelift_codegen::ir::Value;
use glsl::syntax::Expr;

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
        let (_, index_ty) = ctx.translate_expr_typed(index_expr)?;
        if index_ty != GlslType::Int {
            return Err(GlslError::new(ErrorCode::E0106, "index must be int")
                .with_location(source_span_to_location(span)));
        }

        // Extract compile-time constant index
        // TODO: Support runtime indices
        let index = if let Expr::IntConst(n, _) = index_expr.as_ref() {
            let n = *n as usize;
            n
        } else {
            return Err(GlslError::new(
                ErrorCode::E0400,
                "indexing with variable index not yet implemented",
            )
            .with_location(source_span_to_location(span))
            .with_note("only compile-time constant indices are supported"));
        };

        if current_ty.is_matrix() {
            // Matrix indexing: mat[col] returns column vector
            let (rows, cols) = current_ty.matrix_dims().unwrap();
            let column_type = current_ty.matrix_column_type().unwrap();

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
        } else if current_ty.is_vector() {
            // Vector indexing: vec[index] returns scalar component
            let component_count = current_ty.component_count().unwrap();

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
            current_vals = vec![current_vals[index]];
            current_ty = base_type;
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

/// Emit component access expression as RValue
///
/// Handles dot notation (e.g., `vec.x`, `vec.xy`) by resolving as LValue then loading.
pub fn emit_component_access_rvalue(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<RValue, GlslError> {
    emit_lvalue_as_rvalue(ctx, expr)
}

/// Emit matrix/vector indexing expression as RValue
///
/// Handles bracket notation (e.g., `vec[0]`, `mat[0][1]`) by resolving as LValue then loading.
pub fn emit_matrix_indexing_rvalue(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<RValue, GlslError> {
    emit_lvalue_as_rvalue(ctx, expr)
}
