//! Helper for building Cranelift function signatures from GLSL types.

use cranelift_codegen::ir::{AbiParam, Signature};
use cranelift_codegen::isa::CallConv;
use crate::semantic::types::Type;
use crate::semantic::functions::Parameter;

/// Builder for Cranelift function signatures from GLSL function signatures.
pub struct SignatureBuilder;

impl SignatureBuilder {
    /// Create a new empty signature with the default calling convention.
    pub fn new() -> Signature {
        Signature::new(CallConv::SystemV)
    }

    /// Build a complete signature from GLSL return type and parameters.
    pub fn build(
        return_type: &Type,
        parameters: &[Parameter],
    ) -> Signature {
        let mut sig = Self::new();
        Self::add_parameters(&mut sig, parameters);
        Self::add_return_type(&mut sig, return_type);
        sig
    }

    /// Add parameters to a signature from GLSL parameters.
    pub fn add_parameters(sig: &mut Signature, parameters: &[Parameter]) {
        for param in parameters {
            Self::add_type_as_params(sig, &param.ty);
        }
    }

    /// Add return type to a signature.
    pub fn add_return_type(sig: &mut Signature, return_type: &Type) {
        if *return_type != Type::Void {
            Self::add_type_as_returns(sig, return_type);
        }
    }

    /// Add a GLSL type as parameters (expanding vectors/matrices into components).
    fn add_type_as_params(sig: &mut Signature, ty: &Type) {
        if ty.is_vector() {
            // Vector: pass each component as separate parameter
            let base_ty = ty.vector_base_type().unwrap();
            let cranelift_ty = base_ty.to_cranelift_type();
            let count = ty.component_count().unwrap();
            for _ in 0..count {
                sig.params.push(AbiParam::new(cranelift_ty));
            }
        } else if ty.is_matrix() {
            // Matrix: pass each element as separate parameter (column-major)
            let element_count = ty.matrix_element_count().unwrap();
            let cranelift_ty = Type::Float.to_cranelift_type();
            for _ in 0..element_count {
                sig.params.push(AbiParam::new(cranelift_ty));
            }
        } else {
            // Scalar: single parameter
            let cranelift_ty = ty.to_cranelift_type();
            sig.params.push(AbiParam::new(cranelift_ty));
        }
    }

    /// Add a GLSL type as return values (expanding vectors/matrices into components).
    fn add_type_as_returns(sig: &mut Signature, ty: &Type) {
        if ty.is_vector() {
            // Vector: return each component
            let base_ty = ty.vector_base_type().unwrap();
            let cranelift_ty = base_ty.to_cranelift_type();
            let count = ty.component_count().unwrap();
            for _ in 0..count {
                sig.returns.push(AbiParam::new(cranelift_ty));
            }
        } else if ty.is_matrix() {
            // Matrix: return each element (column-major)
            let element_count = ty.matrix_element_count().unwrap();
            let cranelift_ty = Type::Float.to_cranelift_type();
            for _ in 0..element_count {
                sig.returns.push(AbiParam::new(cranelift_ty));
            }
        } else {
            // Scalar: single return value
            let cranelift_ty = ty.to_cranelift_type();
            sig.returns.push(AbiParam::new(cranelift_ty));
        }
    }

    /// Count how many Cranelift parameters a GLSL type will expand to.
    pub fn count_parameters(ty: &Type) -> usize {
        if ty.is_vector() {
            ty.component_count().unwrap()
        } else if ty.is_matrix() {
            ty.matrix_element_count().unwrap()
        } else {
            1
        }
    }

    /// Count how many Cranelift return values a GLSL type will expand to.
    pub fn count_returns(ty: &Type) -> usize {
        if ty == &Type::Void {
            0
        } else if ty.is_vector() {
            ty.component_count().unwrap()
        } else if ty.is_matrix() {
            ty.matrix_element_count().unwrap()
        } else {
            1
        }
    }
}

