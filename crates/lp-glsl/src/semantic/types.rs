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

    /// Returns true if this type is numeric (can be used in arithmetic)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    /// Returns true if this type is a scalar (single value)
    pub fn is_scalar(&self) -> bool {
        matches!(self, Type::Bool | Type::Int | Type::Float)
    }

    /// Returns true if this type is a vector
    pub fn is_vector(&self) -> bool {
        matches!(self, 
            Type::Vec2 | Type::Vec3 | Type::Vec4 |
            Type::IVec2 | Type::IVec3 | Type::IVec4 |
            Type::BVec2 | Type::BVec3 | Type::BVec4
        )
    }

    /// Get the base scalar type of a vector (Vec3 → Float, IVec2 → Int)
    pub fn vector_base_type(&self) -> Option<Type> {
        match self {
            Type::Vec2 | Type::Vec3 | Type::Vec4 => Some(Type::Float),
            Type::IVec2 | Type::IVec3 | Type::IVec4 => Some(Type::Int),
            Type::BVec2 | Type::BVec3 | Type::BVec4 => Some(Type::Bool),
            _ => None,
        }
    }

    /// Get number of components (Vec3 → 3, IVec2 → 2)
    pub fn component_count(&self) -> Option<usize> {
        match self {
            Type::Vec2 | Type::IVec2 | Type::BVec2 => Some(2),
            Type::Vec3 | Type::IVec3 | Type::BVec3 => Some(3),
            Type::Vec4 | Type::IVec4 | Type::BVec4 => Some(4),
            _ => None,
        }
    }

    /// Create vector type from base type and count
    pub fn vector_type(base: &Type, count: usize) -> Option<Type> {
        match (base, count) {
            (Type::Float, 2) => Some(Type::Vec2),
            (Type::Float, 3) => Some(Type::Vec3),
            (Type::Float, 4) => Some(Type::Vec4),
            (Type::Int, 2) => Some(Type::IVec2),
            (Type::Int, 3) => Some(Type::IVec3),
            (Type::Int, 4) => Some(Type::IVec4),
            (Type::Bool, 2) => Some(Type::BVec2),
            (Type::Bool, 3) => Some(Type::BVec3),
            (Type::Bool, 4) => Some(Type::BVec4),
            _ => None,
        }
    }

    /// Get the corresponding Cranelift type
    pub fn to_cranelift_type(&self) -> cranelift_codegen::ir::Type {
        match self {
            Type::Bool => cranelift_codegen::ir::types::I8,
            Type::Int => cranelift_codegen::ir::types::I32,
            Type::Float => cranelift_codegen::ir::types::F32,
            Type::Void => panic!("Void type has no Cranelift representation"),
            _ => panic!("Type not yet supported"),
        }
    }
}

