use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::error::{ErrorCode, GlslError, source_span_to_location};
use cranelift_codegen::ir::Value;

use super::binary;
use super::coercion;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_vector_binary(
    ctx: &mut CodegenContext,
    op: &glsl::syntax::BinaryOp,
    lhs_vals: Vec<Value>,
    lhs_ty: &GlslType,
    rhs_vals: Vec<Value>,
    rhs_ty: &GlslType,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    use glsl::syntax::BinaryOp::*;

    // Validate operation is allowed on vectors
    match op {
        Add | Sub | Mult | Div => {}, // allowed
        _ => return Err(GlslError::new(ErrorCode::E0400, format!("operator {:?} not supported on vectors yet", op))),
    }

    // Determine result type and operation mode
    enum VectorOpMode {
        ComponentWise,    // vec + vec
        VectorScalar,     // vec + scalar
        ScalarVector,     // scalar + vec
    }

    let (result_ty, mode) = if lhs_ty.is_vector() && rhs_ty.is_vector() {
        // vec + vec: component-wise, types must match
        if lhs_ty != rhs_ty {
            let mut error = GlslError::new(ErrorCode::E0106, format!(
                "vector operation requires matching types, got {:?} and {:?}",
                lhs_ty, rhs_ty
            ));
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            return Err(error);
        }
        (lhs_ty.clone(), VectorOpMode::ComponentWise)
    } else if lhs_ty.is_vector() && rhs_ty.is_scalar() {
        // vec + scalar: scalar applied to each component
        let vec_base = lhs_ty.vector_base_type().unwrap();
        // Check if scalar can be used with vector base type
        if !rhs_ty.is_numeric() || !vec_base.is_numeric() {
            return Err(GlslError::new(ErrorCode::E0106, format!(
                "cannot use {:?} with {:?}",
                rhs_ty, lhs_ty
            )));
        }
        (lhs_ty.clone(), VectorOpMode::VectorScalar)
    } else if lhs_ty.is_scalar() && rhs_ty.is_vector() {
        // scalar + vec: scalar applied to each component
        let vec_base = rhs_ty.vector_base_type().unwrap();
        if !lhs_ty.is_numeric() || !vec_base.is_numeric() {
            return Err(GlslError::new(ErrorCode::E0400, format!(
                "Cannot use {:?} with {:?}",
                lhs_ty, rhs_ty
            )));
        }
        (rhs_ty.clone(), VectorOpMode::ScalarVector)
    } else {
        unreachable!("translate_vector_binary called with non-vector types");
    };

    let base_ty = result_ty.vector_base_type().unwrap();
    let component_count = result_ty.component_count().unwrap();

    let mut result_vals = Vec::new();
    
    match mode {
        VectorOpMode::ComponentWise => {
            // vec3(a,b,c) + vec3(d,e,f) = vec3(a+d, b+e, c+f)
            for i in 0..component_count {
                let lhs_comp = lhs_vals[i];
                let rhs_comp = rhs_vals[i];
                let result_comp = binary::translate_scalar_binary_op_internal(ctx, op, lhs_comp, rhs_comp, &base_ty)?;
                result_vals.push(result_comp);
            }
        }
        VectorOpMode::VectorScalar => {
            // vec3(a,b,c) * s = vec3(a*s, b*s, c*s)
            let scalar = rhs_vals[0];
            // Coerce scalar to vector base type if needed
            let scalar = coercion::coerce_to_type(ctx, scalar, rhs_ty, &base_ty)?;
            for &comp in &lhs_vals {
                let result_comp = binary::translate_scalar_binary_op_internal(ctx, op, comp, scalar, &base_ty)?;
                result_vals.push(result_comp);
            }
        }
        VectorOpMode::ScalarVector => {
            // s * vec3(a,b,c) = vec3(s*a, s*b, s*c)
            let scalar = lhs_vals[0];
            // Coerce scalar to vector base type if needed
            let scalar = coercion::coerce_to_type(ctx, scalar, lhs_ty, &base_ty)?;
            for &comp in &rhs_vals {
                let result_comp = binary::translate_scalar_binary_op_internal(ctx, op, scalar, comp, &base_ty)?;
                result_vals.push(result_comp);
            }
        }
    }

    Ok((result_vals, result_ty))
}



