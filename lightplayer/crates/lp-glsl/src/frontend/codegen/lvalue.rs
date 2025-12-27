//! LValue abstraction for unified handling of modifiable locations
//!
//! This module provides a unified interface for handling all modifiable locations
//! (variables, vector components, matrix elements, etc.) in a single place,
//! eliminating code duplication across assignment, increment, and decrement operations.

use crate::error::{
    ErrorCode, GlslError, extract_span_from_expr, extract_span_from_identifier,
    source_span_to_location,
};
use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::codegen::rvalue::RValue;
use crate::semantic::types::Type as GlslType;
use cranelift_codegen::ir::Value;
use cranelift_frontend::Variable;
use glsl::syntax::Expr;

use super::expr::component;

use alloc::{format, vec::Vec};

/// Represents a modifiable location (LValue) in GLSL
///
/// This enum abstracts over all possible modifiable locations, allowing
/// unified handling of variables, vector components, matrix elements, etc.
#[derive(Debug, Clone)]
pub enum LValue {
    /// Simple variable: `x`
    Variable { vars: Vec<Variable>, ty: GlslType },
    /// Vector component access: `v.x` or `v.xy`
    Component {
        base_vars: Vec<Variable>,
        base_ty: GlslType,
        indices: Vec<usize>, // Component indices
        result_ty: GlslType,
    },
    /// Matrix element: `m[0][1]` (single scalar)
    MatrixElement {
        base_vars: Vec<Variable>,
        base_ty: GlslType,
        row: usize,
        col: usize,
    },
    /// Matrix column: `m[0]` (vector)
    MatrixColumn {
        base_vars: Vec<Variable>,
        base_ty: GlslType,
        col: usize,
        result_ty: GlslType,
    },
    /// Vector element: `v[0]` (single scalar)
    VectorElement {
        base_vars: Vec<Variable>,
        base_ty: GlslType,
        index: usize, // Component index (0=x, 1=y, 2=z, 3=w)
    },
}

impl LValue {
    /// Get the type of this LValue
    pub fn ty(&self) -> GlslType {
        match self {
            LValue::Variable { ty, .. } => ty.clone(),
            LValue::Component { result_ty, .. } => result_ty.clone(),
            LValue::MatrixElement { .. } => {
                // Matrix element is always float scalar
                GlslType::Float
            }
            LValue::MatrixColumn { result_ty, .. } => result_ty.clone(),
            LValue::VectorElement { base_ty, .. } => {
                // Vector element type is the base type of the vector
                base_ty.vector_base_type().unwrap()
            }
        }
    }
}

/// Compute matrix variable indices for column components
///
/// When accessing components of a matrix column (e.g., `m[0].x`), we need to map
/// the component indices (0=x, 1=y, etc.) to the correct matrix variable indices.
/// For a column `col` and component index `comp_idx`, the matrix variable index is `col * rows + comp_idx`.
fn compute_column_variable_indices(
    base_ty: &GlslType,
    col: usize,
    component_indices: &[usize],
) -> Vec<usize> {
    let (rows, _cols) = base_ty.matrix_dims().unwrap();
    component_indices
        .iter()
        .map(|&comp_idx| col * rows + comp_idx)
        .collect()
}

/// Resolve an expression to an LValue
///
/// Recursively analyzes the expression to determine the modifiable location.
/// Handles nested expressions like `m[0].x` by first resolving `m[0]` then extracting the component.
pub fn resolve_lvalue(ctx: &mut CodegenContext, expr: &Expr) -> Result<LValue, GlslError> {
    match expr {
        Expr::Variable(ident, _span) => {
            let span = extract_span_from_identifier(ident);
            let vars = ctx
                .lookup_variables(&ident.name)
                .ok_or_else(|| {
                    let error = GlslError::undefined_variable(&ident.name)
                        .with_location(source_span_to_location(&span));
                    ctx.add_span_to_error(error, &span)
                })?
                .to_vec();

            let ty = ctx
                .lookup_variable_type(&ident.name)
                .ok_or_else(|| {
                    let error = GlslError::new(
                        ErrorCode::E0400,
                        format!("variable type not found for `{}`", ident.name),
                    )
                    .with_location(source_span_to_location(&span));
                    ctx.add_span_to_error(error, &span)
                })?
                .clone();

            Ok(LValue::Variable { vars, ty })
        }

        Expr::Dot(base_expr, field, _dot_span) => {
            // Recursively resolve the base expression to an LValue
            let base_lvalue = resolve_lvalue(ctx, base_expr)?;

            // Extract component indices from the field name
            let base_ty = match &base_lvalue {
                LValue::Variable { ty, .. } => ty.clone(),
                LValue::Component { result_ty, .. } => result_ty.clone(),
                LValue::MatrixColumn { result_ty, .. } => result_ty.clone(),
                LValue::MatrixElement { .. } | LValue::VectorElement { .. } => {
                    // Can't access components of a scalar
                    let span = extract_span_from_expr(base_expr);
                    return Err(GlslError::new(
                        ErrorCode::E0112,
                        "component access on scalar value",
                    )
                    .with_location(source_span_to_location(&span)));
                }
            };

            if !base_ty.is_vector() {
                let span = extract_span_from_expr(base_expr);
                return Err(GlslError::new(
                    ErrorCode::E0112,
                    format!("component access on non-vector type: {:?}", base_ty),
                )
                .with_location(source_span_to_location(&span)));
            }

            let field_span = extract_span_from_identifier(field);
            let indices =
                component::parse_vector_swizzle(&field.name, &base_ty, Some(field_span.clone()))?;
            let base_component_ty = base_ty.vector_base_type().unwrap();

            let result_ty = if indices.len() == 1 {
                base_component_ty.clone()
            } else {
                GlslType::vector_type(&base_component_ty, indices.len()).ok_or_else(|| {
                    GlslError::new(
                        ErrorCode::E0400,
                        format!("cannot create vector of size {}", indices.len()),
                    )
                    .with_location(source_span_to_location(&field_span))
                })?
            };

            // Get the base variables and compute indices based on the base LValue type
            match base_lvalue {
                LValue::Variable { vars, .. } => Ok(LValue::Component {
                    base_vars: vars,
                    base_ty,
                    indices,
                    result_ty,
                }),
                LValue::Component { base_vars, .. } => Ok(LValue::Component {
                    base_vars,
                    base_ty,
                    indices,
                    result_ty,
                }),
                LValue::MatrixColumn {
                    base_vars,
                    base_ty: matrix_ty,
                    col,
                    ..
                } => {
                    // When accessing components of a matrix column, we need to map component indices
                    // (0=x, 1=y, etc.) to the correct matrix variable indices.
                    // For column `col` and component index `comp_idx`, the matrix variable index is `col * rows + comp_idx`.
                    let matrix_indices = compute_column_variable_indices(&matrix_ty, col, &indices);
                    Ok(LValue::Component {
                        base_vars,
                        base_ty: matrix_ty.clone(),
                        indices: matrix_indices,
                        result_ty,
                    })
                }
                LValue::MatrixElement { .. } | LValue::VectorElement { .. } => unreachable!(), // Already handled above
            }
        }

        Expr::Bracket(array_expr, array_spec, span) => {
            // Recursively resolve the base expression to an LValue
            let base_lvalue = resolve_lvalue(ctx, array_expr)?;

            // Get base variables and type
            let (base_vars, base_ty) = match base_lvalue {
                LValue::Variable { vars, ty } => (vars, ty),
                LValue::Component {
                    base_vars, base_ty, ..
                } => (base_vars, base_ty),
                LValue::MatrixColumn {
                    base_vars, base_ty, ..
                } => (base_vars, base_ty),
                LValue::MatrixElement {
                    base_vars, base_ty, ..
                } => (base_vars, base_ty),
                LValue::VectorElement {
                    base_vars, base_ty, ..
                } => (base_vars, base_ty),
            };

            use glsl::syntax::ArraySpecifierDimension;
            if array_spec.dimensions.0.is_empty() {
                return Err(
                    GlslError::new(ErrorCode::E0400, "indexing requires explicit index")
                        .with_location(source_span_to_location(span)),
                );
            }

            // Process dimensions one at a time
            let mut current_ty = base_ty.clone();
            let current_vars = base_vars;
            let mut row: Option<usize> = None;
            let mut col: Option<usize> = None;

            for (_dim_idx, dimension) in array_spec.dimensions.0.iter().enumerate() {
                let index_expr = match dimension {
                    ArraySpecifierDimension::ExplicitlySized(expr) => expr,
                    ArraySpecifierDimension::Unsized => {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            "indexing requires explicit index",
                        )
                        .with_location(source_span_to_location(span)));
                    }
                };

                // Evaluate index (must be int)
                let (_, index_ty) = ctx.translate_expr_typed(index_expr)?;
                if index_ty != GlslType::Int {
                    return Err(GlslError::new(ErrorCode::E0106, "index must be int")
                        .with_location(source_span_to_location(span)));
                }

                // Extract compile-time constant index
                // For LValues (writes), we only support constant indices
                // Variable-indexed reads are handled via translate_matrix_indexing()
                let index = if let Expr::IntConst(n, _) = index_expr.as_ref() {
                    *n as usize
                } else {
                    return Err(GlslError::new(
                        ErrorCode::E0400,
                        "variable-indexed writes not yet implemented",
                    )
                    .with_location(source_span_to_location(span))
                    .with_note("only compile-time constant indices are supported for writes"));
                };

                if current_ty.is_matrix() {
                    // Matrix indexing: mat[col] returns column vector
                    let (_rows, cols) = current_ty.matrix_dims().unwrap();

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

                    col = Some(index);
                    current_ty = current_ty.matrix_column_type().unwrap();
                    // Don't update current_vars here - we'll use them for the final LValue
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

                    // If we already have a column, this is a matrix element access
                    if col.is_some() {
                        row = Some(index);
                        current_ty = current_ty.vector_base_type().unwrap();
                    } else {
                        // This is vector element access: v[0] -> scalar
                        return Ok(LValue::VectorElement {
                            base_vars: current_vars,
                            base_ty: base_ty.clone(),
                            index,
                        });
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

            // Determine the final LValue type based on what we found
            match (row, col) {
                (Some(row), Some(col)) => {
                    // Matrix element: m[col][row]
                    Ok(LValue::MatrixElement {
                        base_vars: current_vars,
                        base_ty: base_ty.clone(),
                        row,
                        col,
                    })
                }
                (None, Some(col)) => {
                    // Matrix column: m[col]
                    let column_ty = base_ty.matrix_column_type().unwrap();
                    Ok(LValue::MatrixColumn {
                        base_vars: current_vars,
                        base_ty: base_ty.clone(),
                        col,
                        result_ty: column_ty,
                    })
                }
                _ => {
                    // Shouldn't happen, but handle gracefully
                    Err(GlslError::new(ErrorCode::E0400, "invalid indexing pattern")
                        .with_location(source_span_to_location(span)))
                }
            }
        }

        _ => {
            let span = extract_span_from_expr(expr);
            Err(GlslError::new(
                ErrorCode::E0115,
                "expression is not a valid LValue (must be variable, component access, or matrix element)",
            )
            .with_location(source_span_to_location(&span)))
        }
    }
}

/// Read the current value(s) from an LValue
///
/// Returns the values and their type.
pub fn read_lvalue(
    ctx: &mut CodegenContext,
    lvalue: &LValue,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    // Must be in block to read variables
    ctx.ensure_block()?;

    match lvalue {
        LValue::Variable { vars, ty } => {
            let vals: Vec<Value> = vars.iter().map(|&v| ctx.builder.use_var(v)).collect();
            Ok((vals, ty.clone()))
        }

        LValue::Component {
            base_vars,
            indices,
            result_ty,
            ..
        } => {
            let mut vals = Vec::new();
            for &idx in indices {
                vals.push(ctx.builder.use_var(base_vars[idx]));
            }
            Ok((vals, result_ty.clone()))
        }

        LValue::MatrixElement {
            base_vars,
            base_ty,
            row,
            col,
        } => {
            let (rows, _cols) = base_ty.matrix_dims().unwrap();
            let val = ctx.load_matrix_element(base_vars, *col, *row, rows);
            Ok((vec![val], GlslType::Float)) // Matrix elements are always float
        }

        LValue::MatrixColumn {
            base_vars,
            base_ty,
            col,
            result_ty,
        } => {
            let (rows, _cols) = base_ty.matrix_dims().unwrap();
            let vals = ctx.load_matrix_column(base_vars, *col, rows);
            Ok((vals, result_ty.clone()))
        }

        LValue::VectorElement {
            base_vars,
            base_ty,
            index,
        } => {
            let val = ctx.builder.use_var(base_vars[*index]);
            let base_type = base_ty.vector_base_type().unwrap();
            crate::debug!(
                "read_lvalue VectorElement: base_ty={:?}, base_type={:?}, index={}",
                base_ty,
                base_type,
                index
            );
            Ok((vec![val], base_type))
        }
    }
}

/// Write new value(s) to an LValue
///
/// Validates that the number of values matches the LValue's component count.
pub fn write_lvalue(
    ctx: &mut CodegenContext,
    lvalue: &LValue,
    values: &[Value],
) -> Result<(), GlslError> {
    // Must be in block to write variables
    ctx.ensure_block()?;

    match lvalue {
        LValue::Variable { vars, .. } => {
            if vars.len() != values.len() {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "component count mismatch: {} variables, {} values",
                        vars.len(),
                        values.len()
                    ),
                ));
            }
            for (var, val) in vars.iter().zip(values.iter()) {
                ctx.builder.def_var(*var, *val);
            }
            Ok(())
        }

        LValue::Component {
            base_vars, indices, ..
        } => {
            if indices.len() != values.len() {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "component count mismatch: {} indices, {} values",
                        indices.len(),
                        values.len()
                    ),
                ));
            }
            for (&idx, &val) in indices.iter().zip(values.iter()) {
                ctx.builder.def_var(base_vars[idx], val);
            }
            Ok(())
        }

        LValue::MatrixElement {
            base_vars,
            base_ty,
            row,
            col,
        } => {
            if values.len() != 1 {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("matrix element requires 1 value, got {}", values.len()),
                ));
            }
            let (rows, _cols) = base_ty.matrix_dims().unwrap();
            ctx.store_matrix_element(base_vars, *col, *row, rows, values[0]);
            Ok(())
        }

        LValue::MatrixColumn {
            base_vars,
            base_ty,
            col,
            result_ty,
        } => {
            let (rows, _cols) = base_ty.matrix_dims().unwrap();
            let expected_count = result_ty.component_count().unwrap();
            if values.len() != expected_count {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!(
                        "matrix column requires {} values, got {}",
                        expected_count,
                        values.len()
                    ),
                ));
            }
            for (row_idx, &val) in values.iter().enumerate() {
                ctx.store_matrix_element(base_vars, *col, row_idx, rows, val);
            }
            Ok(())
        }

        LValue::VectorElement {
            base_vars, index, ..
        } => {
            if values.len() != 1 {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    format!("vector element requires 1 value, got {}", values.len()),
                ));
            }
            ctx.builder.def_var(base_vars[*index], values[0]);
            Ok(())
        }
    }
}

/// Common pattern: resolve expression as LValue, then load it as RValue
///
/// This pattern is used for Variable, Dot, and Bracket expressions.
/// First resolves the expression to a modifiable location (LValue),
/// then reads the current value(s) from that location.
pub fn emit_lvalue_as_rvalue(ctx: &mut CodegenContext, expr: &Expr) -> Result<RValue, GlslError> {
    let lvalue = resolve_lvalue(ctx, expr)?;
    let (vals, ty) = read_lvalue(ctx, &lvalue)?;
    Ok(RValue::from_aggregate(vals, ty))
}
