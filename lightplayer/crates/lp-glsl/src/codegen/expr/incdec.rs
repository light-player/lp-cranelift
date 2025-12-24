//! Increment/decrement expression code generation
//!
//! Handles pre-increment (++i), pre-decrement (--i), post-increment (i++),
//! and post-decrement (i--) operations on scalars, vectors, matrices, and vector components.
//! Implements GLSL spec: operators.adoc:856-869

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::{infer_postinc_result_type, infer_postdec_result_type, infer_preinc_result_type, infer_predec_result_type};
use crate::error::{ErrorCode, GlslError, extract_span_from_identifier, source_span_to_location};
use glsl::syntax::Expr;
use cranelift_codegen::ir::{types, Value, InstBuilder};

use super::component;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

/// Translate pre-increment expression (++i)
pub fn translate_preinc(
    ctx: &mut CodegenContext,
    operand: &Expr,
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    translate_incdec(ctx, operand, span, true, false, true)
}

/// Translate pre-decrement expression (--i)
pub fn translate_predec(
    ctx: &mut CodegenContext,
    operand: &Expr,
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    translate_incdec(ctx, operand, span, false, true, true)
}

/// Translate post-increment expression (i++)
pub fn translate_postinc(
    ctx: &mut CodegenContext,
    operand: &Expr,
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    translate_incdec(ctx, operand, span, true, false, false)
}

/// Translate post-decrement expression (i--)
pub fn translate_postdec(
    ctx: &mut CodegenContext,
    operand: &Expr,
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    translate_incdec(ctx, operand, span, false, true, false)
}

/// Common implementation for increment/decrement operations
fn translate_incdec(
    ctx: &mut CodegenContext,
    operand: &Expr,
    span: glsl::syntax::SourceSpan,
    is_increment: bool,
    is_decrement: bool,
    is_pre: bool,
) -> Result<(Vec<Value>, GlslType), GlslError> {

    // Handle variable references and component access
    match operand {
        Expr::Variable(ident, _span) => {
            translate_variable_incdec(ctx, &ident.name, span, is_increment, is_decrement, is_pre)
        }
        Expr::Dot(vec_expr, field, _span) => {
            translate_component_incdec(ctx, vec_expr, field, span, is_increment, is_decrement, is_pre)
        }
        _ => Err(GlslError::new(ErrorCode::E0400, "increment/decrement only supported on variables, vector components, and matrices")
            .with_location(source_span_to_location(&span))),
    }
}

/// Translate increment/decrement on a variable (scalar, vector, or matrix)
fn translate_variable_incdec(
    ctx: &mut CodegenContext,
    var_name: &str,
    span: glsl::syntax::SourceSpan,
    is_increment: bool,
    is_decrement: bool,
    is_pre: bool,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    // Get variable info
    let vars = ctx.lookup_variables(var_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable `{}` not found", var_name)))?;

    let var_ty = ctx.lookup_variable_type(var_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable type not found for `{}`", var_name)))?;

    // Type check the operation
    let result_ty = match (is_increment, is_decrement, is_pre) {
        (true, false, true) => infer_preinc_result_type(var_ty, span)?,
        (false, true, true) => infer_predec_result_type(var_ty, span)?,
        (true, false, false) => infer_postinc_result_type(var_ty, span)?,
        (false, true, false) => infer_postdec_result_type(var_ty, span)?,
        _ => unreachable!(),
    };

    // Clone the variables to avoid borrowing issues
    let vars = vars.to_vec();
    let base_ty = if var_ty.is_matrix() {
        GlslType::Float // Matrices are always float
    } else {
        var_ty.vector_base_type().unwrap_or(var_ty.clone())
    };

    let mut old_values = Vec::new();
    let mut new_values = Vec::new();

    // Process each component (works for scalars, vectors, and matrices)
    for var in &vars {
        // Load current value (this is what we return for post-increment/decrement)
        let old_value = ctx.builder.use_var(*var);
        old_values.push(old_value);

        // Compute new value (add or subtract 1)
        let new_value = match base_ty {
            GlslType::Int => {
                let one = ctx.builder.ins().iconst(types::I32, if is_increment { 1 } else { -1 });
                ctx.builder.ins().iadd(old_value, one)
            }
            GlslType::Float => {
                let one = ctx.builder.ins().f32const(1.0);
                if is_increment {
                    ctx.builder.ins().fadd(old_value, one)
                } else if is_decrement {
                    ctx.builder.ins().fsub(old_value, one)
                } else {
                    unreachable!()
                }
            }
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("increment/decrement not supported for type {:?}", base_ty))),
        };

        new_values.push(new_value);
    }

    // Store new values back to variables
    for (var, new_value) in vars.iter().zip(&new_values) {
        ctx.builder.def_var(*var, *new_value);
    }

    // Return values based on pre vs post semantics
    let return_values = if is_pre {
        // Pre-increment/decrement: return new values
        new_values
    } else {
        // Post-increment/decrement: return old values
        old_values
    };
    Ok((return_values, result_ty))
}

/// Translate increment/decrement on a vector component (e.g., v.x++)
fn translate_component_incdec(
    ctx: &mut CodegenContext,
    vec_expr: &Expr,
    field: &glsl::syntax::Identifier,
    span: glsl::syntax::SourceSpan,
    is_increment: bool,
    is_decrement: bool,
    is_pre: bool,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    // For now, only handle simple variable components (like v.x, not complex expressions)
    let var_name = match vec_expr {
        Expr::Variable(ident, _span) => &ident.name,
        _ => return Err(GlslError::new(ErrorCode::E0400, "component increment/decrement only supported on variables for now")
            .with_location(source_span_to_location(&span))),
    };

    // Get the vector variable info
    let vars = ctx.lookup_variables(var_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable `{}` not found", var_name)))?;

    let var_ty = ctx.lookup_variable_type(var_name)
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable type not found for `{}`", var_name)))?;

    if !var_ty.is_vector() {
        return Err(GlslError::new(ErrorCode::E0112, format!("component access on non-vector variable: {}", var_name))
            .with_location(source_span_to_location(&span)));
    }

    // Parse the swizzle to get component indices
    let field_span = extract_span_from_identifier(field);
    let indices = component::parse_vector_swizzle(&field.name, var_ty, Some(field_span.clone()))?;

    if indices.len() != 1 {
        return Err(GlslError::new(ErrorCode::E0400, "increment/decrement only supported on single components (not swizzles)")
            .with_location(source_span_to_location(&span)));
    }

    let component_index = indices[0];
    let base_ty = var_ty.vector_base_type().unwrap();

    // Type check the operation
    let result_ty = match (is_increment, is_decrement, is_pre) {
        (true, false, true) => infer_preinc_result_type(&base_ty, span)?,
        (false, true, true) => infer_predec_result_type(&base_ty, span)?,
        (true, false, false) => infer_postinc_result_type(&base_ty, span)?,
        (false, true, false) => infer_postdec_result_type(&base_ty, span)?,
        _ => unreachable!(),
    };

    // Clone variables to avoid borrowing issues
    let vars = vars.to_vec();

    // Load the current component value
    let old_value = ctx.builder.use_var(vars[component_index]);

    // Compute new value
    let new_value = match base_ty {
        GlslType::Int => {
            let one = ctx.builder.ins().iconst(types::I32, if is_increment { 1 } else { -1 });
            ctx.builder.ins().iadd(old_value, one)
        }
        GlslType::Float => {
            let one = ctx.builder.ins().f32const(1.0);
            if is_increment {
                ctx.builder.ins().fadd(old_value, one)
            } else if is_decrement {
                ctx.builder.ins().fsub(old_value, one)
            } else {
                unreachable!()
            }
        }
        _ => return Err(GlslError::new(ErrorCode::E0400, format!("increment/decrement not supported for component type {:?}", base_ty))),
    };

    // Store new value back to the component
    ctx.builder.def_var(vars[component_index], new_value);

    // Return value based on pre vs post semantics
    let return_value = if is_pre { new_value } else { old_value };
    Ok((vec![return_value], result_ty))
}

