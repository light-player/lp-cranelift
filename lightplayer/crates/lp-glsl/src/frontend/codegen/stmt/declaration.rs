use glsl::syntax::Declaration;

use alloc::{format, vec::Vec};

use crate::error::{ErrorCode, GlslError};
use crate::frontend::codegen::context::CodegenContext;

/// Emit variable declaration statement
pub fn emit_declaration<M: cranelift_module::Module>(ctx: &mut CodegenContext<'_, M>, decl: &Declaration) -> Result<(), GlslError> {
    use glsl::syntax::Declaration;

    match decl {
        Declaration::InitDeclaratorList(list) => {
            // Get base type from type specifier
            let mut ty = parse_type_specifier(ctx, &list.head.ty)?;

            // Handle the head declaration
            if let Some(name) = &list.head.name {
                // Combine array specifier from SingleDeclaration with base type
                // For "int arr[5];", the [5] is in list.head.array_specifier
                if let Some(array_spec) = &list.head.array_specifier {
                    ty = crate::frontend::semantic::type_resolver::apply_array_specifier(&ty, array_spec, Some(name.span.clone()))?;
                }
                
                let vars = ctx.declare_variable(name.name.clone(), ty.clone())?;

                // Handle initializer if present
                // Skip initialization for arrays (Phase 1 doesn't support array initialization)
                if ty.is_array() {
                    if list.head.initializer.is_some() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            "array initialization not yet supported",
                        ));
                    }
                } else if let Some(init) = &list.head.initializer {
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
            // For tail declarations, array specifier is in ArrayedIdentifier.array_spec
            for declarator in &list.tail {
                let mut declarator_ty = ty.clone();
                if let Some(array_spec) = &declarator.ident.array_spec {
                    declarator_ty = crate::frontend::semantic::type_resolver::apply_array_specifier(&ty, array_spec, Some(declarator.ident.ident.span.clone()))?;
                }
                
                let vars = ctx.declare_variable(declarator.ident.ident.name.clone(), declarator_ty)?;

                // Skip initialization for arrays (Phase 1 doesn't support array initialization)
                if ty.is_array() {
                    if declarator.initializer.is_some() {
                        return Err(GlslError::new(
                            ErrorCode::E0400,
                            "array initialization not yet supported",
                        ));
                    }
                } else if let Some(init) = &declarator.initializer {
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
pub fn parse_type_specifier<M: cranelift_module::Module>(
    _ctx: &CodegenContext<'_, M>,
    type_spec: &glsl::syntax::FullySpecifiedType,
) -> Result<crate::frontend::semantic::types::Type, GlslError> {
    // Use unified type parser from type_resolver.rs which handles arrays
    crate::frontend::semantic::type_resolver::parse_type_specifier(&type_spec.ty, None)
}

/// Emit initializer expression (returns values and type)
pub fn emit_initializer<M: cranelift_module::Module>(
    ctx: &mut CodegenContext<'_, M>,
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
        Initializer::Simple(expr) => ctx.emit_expr_typed(expr.as_ref()),
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            "only simple initializers supported",
        )),
    }
}
