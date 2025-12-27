//! Type parsing utilities for converting GLSL AST types to our Type enum

use crate::error::{source_span_to_location, GlslError};
use crate::frontend::semantic::types;

/// Parse GLSL type specifier into our Type enum
pub fn parse_type_specifier(
    ty: &glsl::syntax::TypeSpecifier,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    use glsl::syntax::TypeSpecifierNonArray;

    match &ty.ty {
        TypeSpecifierNonArray::Void => Ok(types::Type::Void),
        TypeSpecifierNonArray::Bool => Ok(types::Type::Bool),
        TypeSpecifierNonArray::Int => Ok(types::Type::Int),
        TypeSpecifierNonArray::Float => Ok(types::Type::Float),
        TypeSpecifierNonArray::Vec2 => Ok(types::Type::Vec2),
        TypeSpecifierNonArray::Vec3 => Ok(types::Type::Vec3),
        TypeSpecifierNonArray::Vec4 => Ok(types::Type::Vec4),
        TypeSpecifierNonArray::IVec2 => Ok(types::Type::IVec2),
        TypeSpecifierNonArray::IVec3 => Ok(types::Type::IVec3),
        TypeSpecifierNonArray::IVec4 => Ok(types::Type::IVec4),
        TypeSpecifierNonArray::BVec2 => Ok(types::Type::BVec2),
        TypeSpecifierNonArray::BVec3 => Ok(types::Type::BVec3),
        TypeSpecifierNonArray::BVec4 => Ok(types::Type::BVec4),
        TypeSpecifierNonArray::Mat2 => Ok(types::Type::Mat2),
        TypeSpecifierNonArray::Mat3 => Ok(types::Type::Mat3),
        TypeSpecifierNonArray::Mat4 => Ok(types::Type::Mat4),
        _ => {
            let mut error = GlslError::unsupported_type(format!("{:?}", ty.ty));
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            Err(error)
        }
    }
}

/// Parse return type from fully specified type
pub fn parse_return_type(
    ty: &glsl::syntax::FullySpecifiedType,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    parse_type_specifier(&ty.ty, span)
}
