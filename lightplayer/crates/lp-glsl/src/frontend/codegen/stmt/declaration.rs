use glsl::syntax::Declaration;

use alloc::{format, vec::Vec};

use crate::error::{ErrorCode, GlslError};
use crate::frontend::codegen::context::CodegenContext;

/// Emit variable declaration statement
pub fn emit_declaration(ctx: &mut CodegenContext, decl: &Declaration) -> Result<(), GlslError> {
    use glsl::syntax::Declaration;

    match decl {
        Declaration::InitDeclaratorList(list) => {
            // Get type from type specifier
            let ty = parse_type_specifier(ctx, &list.head.ty)?;

            // Handle the head declaration
            if let Some(name) = &list.head.name {
                let vars = ctx.declare_variable(name.name.clone(), ty.clone())?;

                // Handle initializer if present
                if let Some(init) = &list.head.initializer {
                    let (init_vals, init_ty) = emit_initializer(ctx, init)?;

                    // Type check (allows implicit conversions)
                    // Extract span from initializer for error reporting
                    let init_span = match init {
                        glsl::syntax::Initializer::Simple(expr) => {
                            crate::error::extract_span_from_expr(expr.as_ref())
                        }
                        _ => glsl::syntax::SourceSpan::unknown(),
                    };
                    match crate::frontend::semantic::type_check::check_assignment(&ty, &init_ty) {
                        Ok(()) => {}
                        Err(mut error) => {
                            if error.location.is_none() {
                                error = error.with_location(crate::error::source_span_to_location(
                                    &init_span,
                                ));
                            }
                            return Err(ctx.add_span_to_error(error, &init_span));
                        }
                    }

                    // Coerce initializer values to match variable type
                    let base_ty = if ty.is_vector() {
                        ty.vector_base_type().unwrap()
                    } else if ty.is_matrix() {
                        crate::frontend::semantic::types::Type::Float
                    } else {
                        ty.clone()
                    };
                    let init_base = if init_ty.is_vector() {
                        init_ty.vector_base_type().unwrap()
                    } else if init_ty.is_matrix() {
                        crate::frontend::semantic::types::Type::Float
                    } else {
                        init_ty.clone()
                    };

                    // Check component counts match
                    if vars.len() != init_vals.len() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!(
                                "component count mismatch: variable has {} components, initializer has {}",
                                vars.len(),
                                init_vals.len()
                            ),
                        ));
                    }

                    // Assign each component with type coercion
                    for (var, val) in vars.iter().zip(&init_vals) {
                        let coerced_val = ctx.coerce_to_type(*val, &init_base, &base_ty)?;
                        ctx.builder.def_var(*var, coerced_val);
                    }
                }
            }

            // Handle tail declarations (same type, different names)
            for declarator in &list.tail {
                let vars = ctx.declare_variable(declarator.ident.ident.name.clone(), ty.clone())?;

                if let Some(init) = &declarator.initializer {
                    let (init_vals, init_ty) = emit_initializer(ctx, init)?;

                    // Type check (allows implicit conversions)
                    crate::frontend::semantic::type_check::check_assignment(&ty, &init_ty)?;

                    // Coerce initializer values to match variable type
                    let base_ty = if ty.is_vector() {
                        ty.vector_base_type().unwrap()
                    } else if ty.is_matrix() {
                        crate::frontend::semantic::types::Type::Float
                    } else {
                        ty.clone()
                    };
                    let init_base = if init_ty.is_vector() {
                        init_ty.vector_base_type().unwrap()
                    } else if init_ty.is_matrix() {
                        crate::frontend::semantic::types::Type::Float
                    } else {
                        init_ty.clone()
                    };

                    // Check component counts match
                    if vars.len() != init_vals.len() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            format!(
                                "component count mismatch: variable has {} components, initializer has {}",
                                vars.len(),
                                init_vals.len()
                            ),
                        ));
                    }

                    // Assign each component with type coercion
                    for (var, val) in vars.iter().zip(&init_vals) {
                        let coerced_val = ctx.coerce_to_type(*val, &init_base, &base_ty)?;
                        ctx.builder.def_var(*var, coerced_val);
                    }
                }
            }

            Ok(())
        }
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            "only variable declarations supported",
        )),
    }
}

/// Parse type specifier from GLSL AST
pub fn parse_type_specifier(
    _ctx: &CodegenContext,
    type_spec: &glsl::syntax::FullySpecifiedType,
) -> Result<crate::frontend::semantic::types::Type, GlslError> {
    use glsl::syntax::TypeSpecifierNonArray;

    match &type_spec.ty.ty {
        TypeSpecifierNonArray::Int => Ok(crate::frontend::semantic::types::Type::Int),
        TypeSpecifierNonArray::Bool => Ok(crate::frontend::semantic::types::Type::Bool),
        TypeSpecifierNonArray::Float => Ok(crate::frontend::semantic::types::Type::Float),
        TypeSpecifierNonArray::Vec2 => Ok(crate::frontend::semantic::types::Type::Vec2),
        TypeSpecifierNonArray::Vec3 => Ok(crate::frontend::semantic::types::Type::Vec3),
        TypeSpecifierNonArray::Vec4 => Ok(crate::frontend::semantic::types::Type::Vec4),
        TypeSpecifierNonArray::IVec2 => Ok(crate::frontend::semantic::types::Type::IVec2),
        TypeSpecifierNonArray::IVec3 => Ok(crate::frontend::semantic::types::Type::IVec3),
        TypeSpecifierNonArray::IVec4 => Ok(crate::frontend::semantic::types::Type::IVec4),
        TypeSpecifierNonArray::BVec2 => Ok(crate::frontend::semantic::types::Type::BVec2),
        TypeSpecifierNonArray::BVec3 => Ok(crate::frontend::semantic::types::Type::BVec3),
        TypeSpecifierNonArray::BVec4 => Ok(crate::frontend::semantic::types::Type::BVec4),
        TypeSpecifierNonArray::Mat2 => Ok(crate::frontend::semantic::types::Type::Mat2),
        TypeSpecifierNonArray::Mat3 => Ok(crate::frontend::semantic::types::Type::Mat3),
        TypeSpecifierNonArray::Mat4 => Ok(crate::frontend::semantic::types::Type::Mat4),
        _ => Err(GlslError::unsupported_type(format!(
            "{:?}",
            type_spec.ty.ty
        ))),
    }
}

/// Emit initializer expression (returns values and type)
pub fn emit_initializer(
    ctx: &mut CodegenContext,
    init: &glsl::syntax::Initializer,
) -> Result<
    (
        Vec<cranelift_codegen::ir::Value>,
        crate::frontend::semantic::types::Type,
    ),
    GlslError,
> {
    use glsl::syntax::Initializer;

    match init {
        Initializer::Simple(expr) => ctx.translate_expr_typed(expr.as_ref()),
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            "only simple initializers supported",
        )),
    }
}
