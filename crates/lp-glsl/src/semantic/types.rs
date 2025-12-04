#[cfg(feature = "std")]
use std::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

/// GLSL type system
/// Phase 1: Only Int and Bool are fully supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Bool,
    Int,

    // Future phases:
    Float,
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    BVec2,
    BVec3,
    BVec4,
    Mat2,
    Mat3,
    Mat4,
    Sampler2D,
    Struct(StructId),
    Array(Box<Type>, usize),
}

pub type StructId = usize;

impl Type {
    /// Returns true if this type is supported in Phase 1
    pub fn is_phase1_supported(&self) -> bool {
        matches!(self, Type::Void | Type::Bool | Type::Int)
    }

    /// Get the corresponding Cranelift type
    pub fn to_cranelift_type(&self) -> cranelift_codegen::ir::Type {
        match self {
            Type::Bool => cranelift_codegen::ir::types::I8,
            Type::Int => cranelift_codegen::ir::types::I32,
            Type::Void => panic!("Void type has no Cranelift representation"),
            _ => panic!("Type not supported in Phase 1"),
        }
    }
}

