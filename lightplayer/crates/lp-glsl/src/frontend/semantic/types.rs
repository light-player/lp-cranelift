use alloc::boxed::Box;
/// GLSL type system
/// Phase 1: Only Int and Bool are fully supported
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    Bool,
    Int,
    UInt,

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
    /// Returns true if this type is numeric (can be used in arithmetic)
    pub fn is_numeric(&self) -> bool {
        match self {
            Type::Int | Type::UInt | Type::Float => true,
            Type::Vec2 | Type::Vec3 | Type::Vec4 | Type::IVec2 | Type::IVec3 | Type::IVec4 => true,
            Type::Mat2 | Type::Mat3 | Type::Mat4 => true,
            _ => false,
        }
    }

    /// Returns true if this type is a scalar (single value)
    pub fn is_scalar(&self) -> bool {
        matches!(self, Type::Bool | Type::Int | Type::UInt | Type::Float)
    }

    /// Returns true if this type is a vector
    pub fn is_vector(&self) -> bool {
        matches!(
            self,
            Type::Vec2
                | Type::Vec3
                | Type::Vec4
                | Type::IVec2
                | Type::IVec3
                | Type::IVec4
                | Type::BVec2
                | Type::BVec3
                | Type::BVec4
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

    /// Returns true if this type is a matrix
    pub fn is_matrix(&self) -> bool {
        matches!(self, Type::Mat2 | Type::Mat3 | Type::Mat4)
    }

    /// Get matrix dimensions (rows, cols) for matrix types
    pub fn matrix_dims(&self) -> Option<(usize, usize)> {
        match self {
            Type::Mat2 => Some((2, 2)),
            Type::Mat3 => Some((3, 3)),
            Type::Mat4 => Some((4, 4)),
            _ => None,
        }
    }

    /// Get the column vector type for a matrix (mat3 → Vec3)
    pub fn matrix_column_type(&self) -> Option<Type> {
        match self {
            Type::Mat2 => Some(Type::Vec2),
            Type::Mat3 => Some(Type::Vec3),
            Type::Mat4 => Some(Type::Vec4),
            _ => None,
        }
    }

    /// Get total number of elements in a matrix (mat3 → 9)
    pub fn matrix_element_count(&self) -> Option<usize> {
        match self {
            Type::Mat2 => Some(4),
            Type::Mat3 => Some(9),
            Type::Mat4 => Some(16),
            _ => None,
        }
    }

    /// Get the corresponding Cranelift type
    ///
    /// Returns an error if the type cannot be converted to a Cranelift type
    /// (e.g., Void type or unsupported types).
    pub fn to_cranelift_type(
        &self,
    ) -> Result<cranelift_codegen::ir::Type, crate::error::GlslError> {
        match self {
            Type::Bool => Ok(cranelift_codegen::ir::types::I8),
            Type::Int => Ok(cranelift_codegen::ir::types::I32),
            Type::UInt => Ok(cranelift_codegen::ir::types::I32),
            Type::Float => Ok(cranelift_codegen::ir::types::F32),
            Type::Void => Err(crate::error::GlslError::new(
                crate::error::ErrorCode::E0109,
                "Void type has no Cranelift representation",
            )),
            Type::Mat2 | Type::Mat3 | Type::Mat4 => {
                // Matrices are stored as arrays of F32 on the stack
                // We return F32 as the base type, actual storage handled in codegen
                Ok(cranelift_codegen::ir::types::F32)
            }
            _ => Err(crate::error::GlslError::new(
                crate::error::ErrorCode::E0109,
                format!("Type not yet supported for codegen: {:?}", self),
            )),
        }
    }
}
