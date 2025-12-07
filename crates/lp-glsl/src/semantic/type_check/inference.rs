//! Expression type inference for GLSL
//! Implements GLSL spec type rules for Phase 3

use crate::error::{ErrorCode, GlslError, extract_span_from_expr, extract_span_from_identifier, source_span_to_location};
use crate::semantic::types::Type;
use crate::semantic::scope::SymbolTable;
use crate::semantic::functions::FunctionRegistry;
use glsl::syntax::Expr;

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use super::operators::{infer_binary_result_type, infer_unary_result_type};
use super::constructors::{check_vector_constructor_with_span, check_matrix_constructor, is_vector_type_name, is_matrix_type_name};
use super::swizzle::parse_swizzle_length;

/// Infer the result type of an expression
pub fn infer_expr_type(
    expr: &Expr,
    symbols: &SymbolTable,
) -> Result<Type, GlslError> {
    infer_expr_type_with_registry(expr, symbols, None)
}

/// Infer the result type of an expression with optional function registry
pub fn infer_expr_type_with_registry(
    expr: &Expr,
    symbols: &SymbolTable,
    func_registry: Option<&FunctionRegistry>,
) -> Result<Type, GlslError> {
    match expr {
        Expr::IntConst(_, _) => Ok(Type::Int),
        Expr::FloatConst(_, _) => Ok(Type::Float),
        Expr::BoolConst(_, _) => Ok(Type::Bool),
        Expr::DoubleConst(_, _) => Ok(Type::Float), // Treat as float for now

        Expr::Variable(ident, _span) => {
            let span = extract_span_from_identifier(ident);
            let var = symbols.lookup_variable(&ident.name)
                .ok_or_else(|| {
                    GlslError::undefined_variable(&ident.name)
                        .with_location(source_span_to_location(&span))
                        .with_note(format!("variable `{}` is not defined in this scope", ident.name))
                })?;
            Ok(var.ty.clone())
        }

        Expr::Binary(op, lhs, rhs, span) => {
            let lhs_ty = infer_expr_type_with_registry(lhs, symbols, func_registry)?;
            let rhs_ty = infer_expr_type_with_registry(rhs, symbols, func_registry)?;
            infer_binary_result_type(op, &lhs_ty, &rhs_ty, span.clone())
        }

        Expr::Unary(op, expr, span) => {
            let expr_ty = infer_expr_type_with_registry(expr, symbols, func_registry)?;
            infer_unary_result_type(op, &expr_ty, span.clone())
        }

        Expr::Assignment(lhs, _op, _rhs, _span) => {
            // Assignment result has same type as LHS
            infer_expr_type_with_registry(lhs, symbols, func_registry)
        }

        Expr::Dot(expr, field, dot_span) => {
            // Component access (swizzle) - infer type of base expression
            let base_ty = infer_expr_type_with_registry(expr, symbols, func_registry)?;
            
            if !base_ty.is_vector() {
                let span = extract_span_from_expr(expr);
                return Err(GlslError::new(
                    ErrorCode::E0112,
                    format!("component access on non-vector type: {:?}", base_ty)
                )
                .with_location(source_span_to_location(&span)));
            }
            
            // Parse swizzle to determine result type
            let component_count = base_ty.component_count().unwrap();
            let dot_span_clone = dot_span.clone();
            let swizzle_len = parse_swizzle_length(&field.name, component_count)
                .map_err(|mut e| {
                    // Add location if not already present
                    if e.location.is_none() {
                        e = e.with_location(source_span_to_location(&dot_span_clone));
                    }
                    e
                })?;
            let base_scalar_ty = base_ty.vector_base_type().unwrap();
            
            if swizzle_len == 1 {
                // Single component: return scalar
                Ok(base_scalar_ty)
            } else {
                // Multiple components: return vector
                Type::vector_type(&base_scalar_ty, swizzle_len)
                    .ok_or_else(|| {
                        let mut error = GlslError::new(
                            ErrorCode::E0113,
                            format!("invalid swizzle length: {}", swizzle_len)
                        );
                        if error.location.is_none() {
                            error = error.with_location(source_span_to_location(&dot_span));
                        }
                        error
                    })
            }
        }

        Expr::FunCall(func_ident, args, span) => {
            // Extract function name
            let func_name = match func_ident {
                glsl::syntax::FunIdentifier::Identifier(ident) => &ident.name,
                _ => {
                    let span = extract_span_from_expr(expr);
                    return Err(GlslError::new(
                        ErrorCode::E0112,
                        "complex function identifiers not yet supported"
                    )
                    .with_location(source_span_to_location(&span)));
                }
            };

            // Infer argument types
            let mut arg_types = Vec::new();
            for arg in args {
                arg_types.push(infer_expr_type_with_registry(arg, symbols, func_registry)?);
            }

            // Check if it's a type constructor (must come before function lookup)
            if is_vector_type_name(func_name) {
                return check_vector_constructor_with_span(func_name, &arg_types, Some(span.clone()));
            }
            
            if is_matrix_type_name(func_name) {
                return check_matrix_constructor(func_name, &arg_types);
            }

            // Check if it's a built-in function
            if crate::semantic::builtins::is_builtin_function(func_name) {
                match crate::semantic::builtins::check_builtin_call(func_name, &arg_types) {
                    Ok(return_type) => Ok(return_type),
                    Err(err_msg) => {
                        Err(GlslError::new(
                            ErrorCode::E0114,
                            err_msg,
                        )
                        .with_location(source_span_to_location(span)))
                    }
                }
            } else if let Some(registry) = func_registry {
                // User-defined function
                let span_clone = span.clone();
                let func_sig = registry.lookup_function(func_name, &arg_types)
                    .map_err(|mut e| {
                        // Add location if not already present
                        if e.location.is_none() {
                            e = e.with_location(source_span_to_location(&span_clone));
                        }
                        e
                    })?;
                Ok(func_sig.return_type.clone())
            } else {
                // No function registry available - can't validate function calls
                let span = extract_span_from_expr(expr);
                Err(GlslError::new(
                    ErrorCode::E0112,
                    format!("cannot infer type for function call `{}` without function registry", func_name)
                )
                .with_location(source_span_to_location(&span)))
            }
        }

        _ => {
            let span = extract_span_from_expr(expr);
            Err(GlslError::new(
                ErrorCode::E0112,
                format!("cannot infer type for expression: {:?}", expr)
            )
            .with_location(source_span_to_location(&span)))
        },
    }
}

