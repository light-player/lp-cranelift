use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::{check_vector_constructor_with_span, check_matrix_constructor};
use crate::error::{GlslError, source_span_to_location};
use glsl::syntax::Expr;
use cranelift_codegen::ir::InstBuilder;

use super::coercion;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_vector_constructor(
    ctx: &mut CodegenContext,
    type_name: &str,
    args: &[Expr],
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    // Translate all arguments
    let mut arg_vals: Vec<Vec<cranelift_codegen::ir::Value>> = Vec::new();
    let mut arg_types: Vec<GlslType> = Vec::new();
    
    for arg in args {
        let (vals, ty) = ctx.translate_expr_typed(arg)?;
        arg_vals.push(vals);
        arg_types.push(ty);
    }

    // Type check constructor
    let result_type = match check_vector_constructor_with_span(type_name, &arg_types, Some(span.clone())) {
        Ok(ty) => ty,
        Err(mut error) => {
            // Ensure error has location and span_text
            if error.location.is_none() {
                error = error.with_location(source_span_to_location(&span));
            }
            return Err(ctx.add_span_to_error(error, &span));
        }
    };
    let base_type = result_type.vector_base_type().unwrap();
    let component_count = result_type.component_count().unwrap();

    // Generate component values
    let mut components = Vec::new();

    // Case 1: Single scalar broadcast
    if arg_types.len() == 1 && arg_types[0].is_scalar() {
        let scalar = arg_vals[0][0];
        let coerced = coercion::coerce_to_type(ctx, scalar, &arg_types[0], &base_type)?;
        for _ in 0..component_count {
            components.push(coerced);
        }
    }
    // Case 2: Single vector conversion
    else if arg_types.len() == 1 && arg_types[0].is_vector() {
        let src_base = arg_types[0].vector_base_type().unwrap();
        for val in &arg_vals[0] {
            components.push(coercion::coerce_to_type(ctx, *val, &src_base, &base_type)?);
        }
    }
    // Case 3: Concatenation
    else {
        for (vals, ty) in arg_vals.iter().zip(&arg_types) {
            let arg_base = if ty.is_vector() {
                ty.vector_base_type().unwrap()
            } else {
                ty.clone()
            };
            
            for &val in vals {
                components.push(coercion::coerce_to_type(ctx, val, &arg_base, &base_type)?);
            }
        }
    }

    Ok((components, result_type))
}

/// Translate matrix constructor
/// Implements GLSL spec: variables.adoc:72-97
pub fn translate_matrix_constructor(
    ctx: &mut CodegenContext,
    type_name: &str,
    args: &[Expr],
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    // Translate all arguments
    let mut arg_vals: Vec<Vec<cranelift_codegen::ir::Value>> = Vec::new();
    let mut arg_types: Vec<GlslType> = Vec::new();
    
    for arg in args {
        let (vals, ty) = ctx.translate_expr_typed(arg)?;
        arg_vals.push(vals);
        arg_types.push(ty);
    }

    // Type check constructor
    let result_type = check_matrix_constructor(type_name, &arg_types)?;
    let (rows, cols) = result_type.matrix_dims().unwrap();
    let element_count = rows * cols;

    // Allocate temporary variables for matrix elements
    let mut matrix_vars = Vec::new();
    for _ in 0..element_count {
        let var = ctx.builder.declare_var(cranelift_codegen::ir::types::F32);
        matrix_vars.push(var);
    }

    // Case 1: Single scalar - identity matrix (diagonal = scalar, rest = 0.0)
    if arg_types.len() == 1 && arg_types[0].is_scalar() {
        let scalar = arg_vals[0][0];
        let scalar_float = coercion::coerce_to_type(ctx, scalar, &arg_types[0], &GlslType::Float)?;
        let zero = ctx.builder.ins().f32const(0.0);
        
        for row in 0..rows {
            for col in 0..cols {
                let value = if row == col { scalar_float } else { zero };
                ctx.builder.def_var(matrix_vars[col * rows + row], value);
            }
        }
    }
    // Case 2: Column vectors - one vector per column
    else if arg_types.len() == cols {
        for (col, (vals, ty)) in arg_vals.iter().zip(&arg_types).enumerate() {
            let vec_base = ty.vector_base_type().unwrap();
            for (row, &val) in vals.iter().enumerate() {
                let float_val = coercion::coerce_to_type(ctx, val, &vec_base, &GlslType::Float)?;
                ctx.builder.def_var(matrix_vars[col * rows + row], float_val);
            }
        }
    }
    // Case 3: Single matrix - conversion between matrix sizes
    else if arg_types.len() == 1 && arg_types[0].is_matrix() {
        let src_matrix_vals = &arg_vals[0];
        let src_ty = &arg_types[0];
        let (src_rows, src_cols) = src_ty.matrix_dims().unwrap();

        // Copy elements from source matrix, padding with identity/truncating as needed
        for col in 0..cols {
            for row in 0..rows {
                let value = if col < src_cols && row < src_rows {
                    // Copy from source matrix
                    let src_idx = col * src_rows + row;
                    src_matrix_vals[src_idx]
                } else if col == row {
                    // Identity padding (diagonal = 1.0)
                    ctx.builder.ins().f32const(1.0)
                } else {
                    // Off-diagonal padding = 0.0
                    ctx.builder.ins().f32const(0.0)
                };
                let var_idx = col * rows + row;
                ctx.builder.def_var(matrix_vars[var_idx], value);
            }
        }
    }
    // Case 4: Mixed scalars - column-major order
    else {
        for col in 0..cols {
            for row in 0..rows {
                let scalar_index = col * rows + row;
                let scalar = arg_vals[scalar_index][0];
                let scalar_ty = &arg_types[scalar_index];
                let float_val = coercion::coerce_to_type(ctx, scalar, scalar_ty, &GlslType::Float)?;
                let var_idx = col * rows + row;
                ctx.builder.def_var(matrix_vars[var_idx], float_val);
            }
        }
    }

    // Return all matrix element values
    let mut result_vals = Vec::new();
    for &var in &matrix_vars {
        result_vals.push(ctx.builder.use_var(var));
    }

    Ok((result_vals, result_type))
}

