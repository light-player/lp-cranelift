//! Type parsing utilities for converting GLSL AST types to our Type enum

use alloc::boxed::Box;
use crate::error::{GlslError, source_span_to_location};
use crate::frontend::semantic::types;

/// Parse GLSL type specifier into our Type enum
pub fn parse_type_specifier(
    ty: &glsl::syntax::TypeSpecifier,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    use glsl::syntax::{TypeSpecifierNonArray, ArraySpecifierDimension};

    // Parse base type
    let base_type = match &ty.ty {
        TypeSpecifierNonArray::Void => types::Type::Void,
        TypeSpecifierNonArray::Bool => types::Type::Bool,
        TypeSpecifierNonArray::Int => types::Type::Int,
        TypeSpecifierNonArray::UInt => types::Type::UInt,
        TypeSpecifierNonArray::Float => types::Type::Float,
        TypeSpecifierNonArray::Vec2 => types::Type::Vec2,
        TypeSpecifierNonArray::Vec3 => types::Type::Vec3,
        TypeSpecifierNonArray::Vec4 => types::Type::Vec4,
        TypeSpecifierNonArray::IVec2 => types::Type::IVec2,
        TypeSpecifierNonArray::IVec3 => types::Type::IVec3,
        TypeSpecifierNonArray::IVec4 => types::Type::IVec4,
        TypeSpecifierNonArray::UVec2 => types::Type::UVec2,
        TypeSpecifierNonArray::UVec3 => types::Type::UVec3,
        TypeSpecifierNonArray::UVec4 => types::Type::UVec4,
        TypeSpecifierNonArray::BVec2 => types::Type::BVec2,
        TypeSpecifierNonArray::BVec3 => types::Type::BVec3,
        TypeSpecifierNonArray::BVec4 => types::Type::BVec4,
        TypeSpecifierNonArray::Mat2 => types::Type::Mat2,
        TypeSpecifierNonArray::Mat3 => types::Type::Mat3,
        TypeSpecifierNonArray::Mat4 => types::Type::Mat4,
        _ => {
            let mut error = GlslError::unsupported_type(format!("{:?}", ty.ty));
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            return Err(error);
        }
    };

    // Parse array dimensions (outermost-first)
    // Example: float[5][3] -> Array(Box<Array(Box<Float>, 3)>, 5)
    if let Some(array_spec) = &ty.array_specifier {
        let mut current_type = base_type;
        
        // Process dimensions from outermost to innermost
        for dimension in &array_spec.dimensions.0 {
            let size = match dimension {
                ArraySpecifierDimension::ExplicitlySized(expr) => {
                    // Extract literal integer constant
                    if let glsl::syntax::Expr::IntConst(n, _) = expr.as_ref() {
                        *n as usize
                    } else {
                        let mut error = GlslError::new(
                            crate::error::ErrorCode::E0400,
                            "array size must be a compile-time constant integer",
                        );
                        if let Some(s) = span {
                            error = error.with_location(source_span_to_location(&s));
                        }
                        return Err(error);
                    }
                }
                ArraySpecifierDimension::Unsized => {
                    let mut error = GlslError::new(
                        crate::error::ErrorCode::E0400,
                        "unsized arrays require initializer to infer size",
                    );
                    if let Some(s) = span {
                        error = error.with_location(source_span_to_location(&s));
                    }
                    return Err(error);
                }
            };

            if size == 0 {
                let mut error = GlslError::new(
                    crate::error::ErrorCode::E0400,
                    "array size must be positive",
                );
                if let Some(s) = span {
                    error = error.with_location(source_span_to_location(&s));
                }
                return Err(error);
            }

            // Wrap current type in Array
            current_type = types::Type::Array(Box::new(current_type), size);
        }

        Ok(current_type)
    } else {
        Ok(base_type)
    }
}

/// Parse return type from fully specified type
pub fn parse_return_type(
    ty: &glsl::syntax::FullySpecifiedType,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    parse_type_specifier(&ty.ty, span)
}

/// Apply array specifier to a base type
/// Used when array dimensions are in the declarator (e.g., "int arr[5];")
pub fn apply_array_specifier(
    base_ty: &types::Type,
    array_spec: &glsl::syntax::ArraySpecifier,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<types::Type, GlslError> {
    use glsl::syntax::ArraySpecifierDimension;
    
    let mut current_type = base_ty.clone();
    
    // Process dimensions from outermost to innermost
    for dimension in &array_spec.dimensions.0 {
        let size = match dimension {
            ArraySpecifierDimension::ExplicitlySized(expr) => {
                // Extract literal integer constant
                if let glsl::syntax::Expr::IntConst(n, _) = expr.as_ref() {
                    *n as usize
                } else {
                    let mut error = GlslError::new(
                        crate::error::ErrorCode::E0400,
                        "array size must be a compile-time constant integer",
                    );
                    if let Some(s) = span {
                        error = error.with_location(source_span_to_location(&s));
                    }
                    return Err(error);
                }
            }
            ArraySpecifierDimension::Unsized => {
                let mut error = GlslError::new(
                    crate::error::ErrorCode::E0400,
                    "unsized arrays require initializer to infer size",
                );
                if let Some(s) = span {
                    error = error.with_location(source_span_to_location(&s));
                }
                return Err(error);
            }
        };

        if size == 0 {
            let mut error = GlslError::new(
                crate::error::ErrorCode::E0400,
                "array size must be positive",
            );
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            return Err(error);
        }

        // Wrap current type in Array
        current_type = types::Type::Array(Box::new(current_type), size);
    }

    Ok(current_type)
}
