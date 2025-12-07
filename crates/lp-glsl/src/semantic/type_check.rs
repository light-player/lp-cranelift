//! Type inference and validation for GLSL expressions
//! Implements GLSL spec type rules for Phase 3

use crate::error::{ErrorCode, GlslError, extract_span_from_expr, extract_span_from_identifier, source_span_to_location};
use crate::semantic::types::Type;
use crate::semantic::scope::SymbolTable;
use crate::semantic::functions::FunctionRegistry;
use glsl::syntax::{Expr, BinaryOp, UnaryOp};

#[cfg(feature = "std")]
use std::{format, vec::Vec};
#[cfg(not(feature = "std"))]
use alloc::{format, vec::Vec};

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

/// Infer result type of binary operation (with implicit conversion)
/// Implements GLSL spec: operators.adoc:775-855, operators.adoc:1019-1098 (matrix ops)
pub fn infer_binary_result_type(
    op: &BinaryOp,
    lhs_ty: &Type,
    rhs_ty: &Type,
    span: glsl::syntax::SourceSpan,
) -> Result<Type, GlslError> {
    use BinaryOp::*;

    match op {
        // Arithmetic operators
        Add | Sub | Mult | Div => {
            // Matrix operations
            if lhs_ty.is_matrix() || rhs_ty.is_matrix() {
                return infer_matrix_binary_result_type(op, lhs_ty, rhs_ty, span);
            }
            
            // Vector operations
            if lhs_ty.is_vector() || rhs_ty.is_vector() {
                // Vector + Vector: component-wise, types must match
                if lhs_ty.is_vector() && rhs_ty.is_vector() {
                    if lhs_ty != rhs_ty {
                        return Err(GlslError::new(
                            ErrorCode::E0106,
                            format!("vector operation requires matching types, got {:?} and {:?}", lhs_ty, rhs_ty)
                        )
                        .with_location(source_span_to_location(&span)));
                    }
                    return Ok(lhs_ty.clone());
                }
                
                // Vector + Scalar or Scalar + Vector: result is vector type
                if lhs_ty.is_vector() {
                    let vec_base = lhs_ty.vector_base_type().unwrap();
                    if !rhs_ty.is_numeric() || !vec_base.is_numeric() {
                        return Err(GlslError::new(
                            ErrorCode::E0106,
                            format!("cannot use {:?} with {:?}", rhs_ty, lhs_ty)
                        )
                        .with_location(source_span_to_location(&span)));
                    }
                    return Ok(lhs_ty.clone());
                }
                
                if rhs_ty.is_vector() {
                    let vec_base = rhs_ty.vector_base_type().unwrap();
                    if !lhs_ty.is_numeric() || !vec_base.is_numeric() {
                        return Err(GlslError::new(
                            ErrorCode::E0106,
                            format!("cannot use {:?} with {:?}", lhs_ty, rhs_ty)
                        )
                        .with_location(source_span_to_location(&span)));
                    }
                    return Ok(rhs_ty.clone());
                }
            }
            
            // Scalar operations
            if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
                return Err(GlslError::new(
                    ErrorCode::E0106,
                    format!("arithmetic operator {:?} requires numeric operands", op)
                )
                .with_location(source_span_to_location(&span))
                .with_note(format!("left operand has type `{:?}`, right operand has type `{:?}`", lhs_ty, rhs_ty)));
            }
            // Result type is the promoted type
            Ok(promote_numeric(lhs_ty, rhs_ty))
        }

        // Comparison operators: operands must be compatible, result is bool
        Equal | NonEqual | LT | GT | LTE | GTE => {
            if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
                return Err(GlslError::new(
                    ErrorCode::E0106,
                    format!("comparison operator {:?} requires numeric operands", op)
                )
                .with_location(source_span_to_location(&span))
                .with_note(format!("left operand has type `{:?}`, right operand has type `{:?}`", lhs_ty, rhs_ty)));
            }
            Ok(Type::Bool)
        }

        // Logical operators: must be bool
        And | Or | Xor => {
            if lhs_ty != &Type::Bool || rhs_ty != &Type::Bool {
                return Err(GlslError::new(
                    ErrorCode::E0106,
                    format!("logical operator {:?} requires bool operands", op)
                )
                .with_location(source_span_to_location(&span))
                .with_note(format!("left operand has type `{:?}`, right operand has type `{:?}`", lhs_ty, rhs_ty)));
            }
            Ok(Type::Bool)
        }

        _ => Err(GlslError::new(
            ErrorCode::E0112,
            format!("unsupported binary operator: {:?}", op)
        )
        .with_location(source_span_to_location(&span))),
    }
}

/// Infer result type of unary operation
pub fn infer_unary_result_type(
    op: &UnaryOp,
    operand_ty: &Type,
    span: glsl::syntax::SourceSpan,
) -> Result<Type, GlslError> {
    use UnaryOp::*;

    match op {
        Minus => {
            if !operand_ty.is_numeric() {
                return Err(GlslError::new(
                    ErrorCode::E0112,
                    "unary minus requires numeric operand"
                )
                .with_location(source_span_to_location(&span))
                .with_note(format!("operand has type `{:?}`", operand_ty)));
            }
            Ok(operand_ty.clone())
        }

        Not => {
            if operand_ty != &Type::Bool {
                return Err(GlslError::new(
                    ErrorCode::E0112,
                    "logical NOT requires bool operand"
                )
                .with_location(source_span_to_location(&span))
                .with_note(format!("operand has type `{:?}`", operand_ty)));
            }
            Ok(Type::Bool)
        }

        _ => Err(GlslError::new(
            ErrorCode::E0112,
            format!("unsupported unary operator: {:?}", op)
        )
        .with_location(source_span_to_location(&span))),
    }
}

/// Promote numeric types (GLSL spec implicit conversion rules)
/// Implements GLSL spec: variables.adoc:1182-1229
pub fn promote_numeric(lhs: &Type, rhs: &Type) -> Type {
    match (lhs, rhs) {
        (Type::Int, Type::Int) => Type::Int,
        (Type::Float, Type::Float) => Type::Float,
        (Type::Int, Type::Float) | (Type::Float, Type::Int) => Type::Float,
        // int → float implicit conversion per GLSL spec
        _ => Type::Int, // Fallback (shouldn't reach here after validation)
    }
}

/// Check if implicit conversion is allowed (GLSL spec: variables.adoc:1182-1229)
pub fn can_implicitly_convert(from: &Type, to: &Type) -> bool {
    // Exact match always allowed
    if from == to {
        return true;
    }
    
    // Scalar conversions
    if matches!((from, to), (Type::Int, Type::Float)) {
        return true;
    }
    
    // Vector conversions: same size, compatible base types
    if let (Some(from_base), Some(to_base), Some(from_count), Some(to_count)) = (
        from.vector_base_type(),
        to.vector_base_type(),
        from.component_count(),
        to.component_count(),
    ) {
        if from_count == to_count {
            return can_implicitly_convert(&from_base, &to_base);
        }
    }
    
    false
}

/// Validate assignment types
pub fn check_assignment(lhs_ty: &Type, rhs_ty: &Type) -> Result<(), GlslError> {
    check_assignment_with_span(lhs_ty, rhs_ty, None)
}

/// Validate assignment types with optional span for error location
pub fn check_assignment_with_span(lhs_ty: &Type, rhs_ty: &Type, span: Option<glsl::syntax::SourceSpan>) -> Result<(), GlslError> {
    if !can_implicitly_convert(rhs_ty, lhs_ty) {
        let mut error = GlslError::new(
            ErrorCode::E0102,
            "type mismatch in assignment"
        )
        .with_note(format!("cannot assign value of type `{:?}` to variable of type `{:?}`", rhs_ty, lhs_ty))
        .with_note("help: consider using an explicit type conversion");
        
        if let Some(span) = span {
            error = error.with_location(source_span_to_location(&span));
        }
        
        return Err(error);
    }
    Ok(())
}

/// Validate condition expression type (must be bool)
pub fn check_condition(cond_ty: &Type) -> Result<(), GlslError> {
    if cond_ty != &Type::Bool {
        return Err(GlslError::new(
            ErrorCode::E0107,
            "condition must be bool type"
        )
        .with_note(format!("condition has type `{:?}`, expected `Bool`", cond_ty)));
    }
    Ok(())
}

/// Check vector constructor arguments and infer result type
pub fn check_vector_constructor(
    type_name: &str,
    args: &[Type],
) -> Result<Type, GlslError> {
    check_vector_constructor_with_span(type_name, args, None)
}

/// Check vector constructor arguments and infer result type with optional span
pub fn check_vector_constructor_with_span(
    type_name: &str,
    args: &[Type],
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<Type, GlslError> {
    let result_type = parse_vector_type_name(type_name)?;
    let component_count = result_type.component_count()
        .ok_or_else(|| {
            GlslError::new(ErrorCode::E0112, format!("`{}` is not a vector type", type_name))
        })?;
    let base_type = result_type.vector_base_type().unwrap();

    // Helper to add location to error if span is available
    let span_clone = span.clone();
    let add_location = move |mut error: GlslError| -> GlslError {
        if let Some(ref s) = span_clone {
            if error.location.is_none() {
                error = error.with_location(source_span_to_location(s));
            }
        }
        error
    };

    // Case 1: Single scalar - broadcast to all components
    if args.len() == 1 && args[0].is_scalar() {
        // Check implicit conversion is allowed
        if !can_implicitly_convert(&args[0], &base_type) {
            return Err(add_location(GlslError::new(
                ErrorCode::E0103,
                format!("cannot construct `{}` from `{:?}`", type_name, args[0])
            )
            .with_note("type cannot be implicitly converted")));
        }
        return Ok(result_type);
    }

    // Case 2: Single vector - type conversion
    if args.len() == 1 && args[0].is_vector() {
        if args[0].component_count() != Some(component_count) {
            return Err(add_location(GlslError::new(
                ErrorCode::E0115,
                format!("cannot construct `{}` from `{:?}`", type_name, args[0])
            )
            .with_note(format!("expected {} components, found {}", component_count, args[0].component_count().unwrap()))));
        }
        // Check base type conversion is allowed
        let src_base = args[0].vector_base_type().unwrap();
        if !can_implicitly_convert(&src_base, &base_type) {
            return Err(add_location(GlslError::new(
                ErrorCode::E0103,
                format!("cannot construct `{}` from `{:?}`", type_name, args[0])
            )
            .with_note("component type cannot be implicitly converted")));
        }
        return Ok(result_type);
    }

    // Case 3: Multiple arguments - concatenation
    let total_components = count_total_components(args)?;
    if total_components != component_count {
        return Err(add_location(GlslError::new(
            ErrorCode::E0115,
            format!("`{}` constructor has wrong number of components", type_name)
        )
        .with_note(format!("expected {} components, found {}", component_count, total_components))));
    }

    // Validate each argument can convert to base type
    for arg in args {
        let arg_base = if arg.is_vector() {
            arg.vector_base_type().unwrap()
        } else {
            arg.clone()
        };
        
        if !can_implicitly_convert(&arg_base, &base_type) {
            return Err(add_location(GlslError::new(
                ErrorCode::E0103,
                format!("cannot use `{:?}` in `{}` constructor", arg, type_name)
            )
            .with_note("component type cannot be implicitly converted")));
        }
    }

    Ok(result_type)
}

fn parse_vector_type_name(name: &str) -> Result<Type, GlslError> {
    match name {
        "vec2" => Ok(Type::Vec2),
        "vec3" => Ok(Type::Vec3),
        "vec4" => Ok(Type::Vec4),
        "ivec2" => Ok(Type::IVec2),
        "ivec3" => Ok(Type::IVec3),
        "ivec4" => Ok(Type::IVec4),
        "bvec2" => Ok(Type::BVec2),
        "bvec3" => Ok(Type::BVec3),
        "bvec4" => Ok(Type::BVec4),
        _ => Err(GlslError::unsupported_type(name)),
    }
}

fn count_total_components(args: &[Type]) -> Result<usize, GlslError> {
    let mut total = 0;
    for arg in args {
        if let Some(count) = arg.component_count() {
            total += count;
        } else if arg.is_scalar() {
            total += 1;
        } else {
            return Err(GlslError::new(
                ErrorCode::E0112,
                format!("invalid constructor argument: `{:?}`", arg)
            ));
        }
    }
    Ok(total)
}

/// Check if a name is a vector type constructor
pub fn is_vector_type_name(name: &str) -> bool {
    matches!(name, 
        "vec2" | "vec3" | "vec4" |
        "ivec2" | "ivec3" | "ivec4" |
        "bvec2" | "bvec3" | "bvec4"
    )
}

/// Check if a name is a matrix type constructor
pub fn is_matrix_type_name(name: &str) -> bool {
    matches!(name, "mat2" | "mat3" | "mat4")
}

/// Parse matrix type name to Type
fn parse_matrix_type_name(name: &str) -> Result<Type, GlslError> {
    match name {
        "mat2" => Ok(Type::Mat2),
        "mat3" => Ok(Type::Mat3),
        "mat4" => Ok(Type::Mat4),
        _ => Err(GlslError::unsupported_type(name)),
    }
}

/// Check matrix constructor arguments and infer result type
/// Implements GLSL spec: variables.adoc:72-97
pub fn check_matrix_constructor(
    type_name: &str,
    args: &[Type],
) -> Result<Type, GlslError> {
    let result_type = parse_matrix_type_name(type_name)?;
    let (rows, cols) = result_type.matrix_dims()
        .ok_or_else(|| {
            GlslError::new(ErrorCode::E0112, format!("`{}` is not a matrix type", type_name))
        })?;
    let element_count = rows * cols;

    // Case 1: Single scalar - identity matrix (diagonal = scalar, rest = 0.0)
    if args.len() == 1 && args[0].is_scalar() {
        if !can_implicitly_convert(&args[0], &Type::Float) {
            return Err(GlslError::new(
                ErrorCode::E0103,
                format!("cannot construct `{}` from `{:?}`", type_name, args[0])
            )
            .with_note("matrix constructor requires float scalar for identity"));
        }
        return Ok(result_type);
    }

    // Case 2: Column vectors - one vector per column
    if args.len() == cols {
        // Check all args are vectors of correct type
        for (i, arg) in args.iter().enumerate() {
            if !arg.is_vector() {
                return Err(GlslError::new(
                    ErrorCode::E0103,
                    format!("matrix column {} must be a vector, got `{:?}`", i, arg)
                ));
            }
            if arg.component_count() != Some(rows) {
                return Err(GlslError::new(
                    ErrorCode::E0115,
                    format!("matrix column {} has wrong size: expected {} components, got {}", 
                        i, rows, arg.component_count().unwrap_or(0))
                ));
            }
            // Check base type can convert to float
            let arg_base = arg.vector_base_type().unwrap();
            if !can_implicitly_convert(&arg_base, &Type::Float) {
                return Err(GlslError::new(
                    ErrorCode::E0103,
                    format!("matrix column {} has incompatible base type: `{:?}`", i, arg_base)
                ));
            }
        }
        return Ok(result_type);
    }

    // Case 3: Mixed scalars - column-major order
    if args.len() == element_count {
        // Check all args are scalars that can convert to float
        for (i, arg) in args.iter().enumerate() {
            if !arg.is_scalar() {
                return Err(GlslError::new(
                    ErrorCode::E0103,
                    format!("matrix element {} must be a scalar, got `{:?}`", i, arg)
                ));
            }
            if !can_implicitly_convert(arg, &Type::Float) {
                return Err(GlslError::new(
                    ErrorCode::E0103,
                    format!("matrix element {} cannot be converted to float: `{:?}`", i, arg)
                ));
            }
        }
        return Ok(result_type);
    }

    // Wrong number of arguments
    Err(GlslError::new(
        ErrorCode::E0115,
        format!("`{}` constructor has wrong number of arguments", type_name)
    )
    .with_note(format!("expected 1 (identity), {} (columns), or {} (scalars), found {}", 
        cols, element_count, args.len())))
}

/// Infer result type of matrix binary operation
/// Implements GLSL spec: operators.adoc:1019-1098
fn infer_matrix_binary_result_type(
    op: &BinaryOp,
    lhs_ty: &Type,
    rhs_ty: &Type,
    span: glsl::syntax::SourceSpan,
) -> Result<Type, GlslError> {
    use BinaryOp::*;

    match op {
        // Matrix + Matrix: component-wise addition (same dimensions)
        Add => {
            if lhs_ty.is_matrix() && rhs_ty.is_matrix() {
                if lhs_ty != rhs_ty {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        "matrix addition requires matching matrix types"
                    )
                    .with_location(source_span_to_location(&span))
                    .with_note(format!("left operand: `{:?}`, right operand: `{:?}`", lhs_ty, rhs_ty)));
                }
                return Ok(lhs_ty.clone());
            }
            Err(GlslError::new(
                ErrorCode::E0106,
                "matrix addition requires both operands to be matrices"
            )
            .with_location(source_span_to_location(&span)))
        }

        // Matrix - Matrix: component-wise subtraction (same dimensions)
        Sub => {
            if lhs_ty.is_matrix() && rhs_ty.is_matrix() {
                if lhs_ty != rhs_ty {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        "matrix subtraction requires matching matrix types"
                    )
                    .with_location(source_span_to_location(&span))
                    .with_note(format!("left operand: `{:?}`, right operand: `{:?}`", lhs_ty, rhs_ty)));
                }
                return Ok(lhs_ty.clone());
            }
            Err(GlslError::new(
                ErrorCode::E0106,
                "matrix subtraction requires both operands to be matrices"
            )
            .with_location(source_span_to_location(&span)))
        }

        // Matrix multiplication
        Mult => {
            // Matrix × Scalar: component-wise multiplication
            if lhs_ty.is_matrix() && rhs_ty.is_scalar() {
                if !rhs_ty.is_numeric() {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        "matrix × scalar requires numeric scalar"
                    )
                    .with_location(source_span_to_location(&span)));
                }
                return Ok(lhs_ty.clone());
            }

            // Scalar × Matrix: component-wise multiplication
            if lhs_ty.is_scalar() && rhs_ty.is_matrix() {
                if !lhs_ty.is_numeric() {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        "scalar × matrix requires numeric scalar"
                    )
                    .with_location(source_span_to_location(&span)));
                }
                return Ok(rhs_ty.clone());
            }

            // Matrix × Vector: linear algebra multiplication
            if lhs_ty.is_matrix() && rhs_ty.is_vector() {
                let (rows, cols) = lhs_ty.matrix_dims().unwrap();
                let vec_size = rhs_ty.component_count().unwrap();
                
                if cols != vec_size {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        format!("matrix × vector dimension mismatch: {}×{} matrix requires {}-component vector", 
                            rows, cols, cols)
                    )
                    .with_location(source_span_to_location(&span))
                    .with_note(format!("got {}-component vector", vec_size)));
                }
                // Result is a vector with same number of components as matrix rows
                return Ok(lhs_ty.matrix_column_type().unwrap());
            }

            // Vector × Matrix: linear algebra multiplication
            if lhs_ty.is_vector() && rhs_ty.is_matrix() {
                let vec_size = lhs_ty.component_count().unwrap();
                let (rows, cols) = rhs_ty.matrix_dims().unwrap();
                
                if vec_size != rows {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        format!("vector × matrix dimension mismatch: {}-component vector requires {}×{} matrix", 
                            vec_size, rows, cols)
                    )
                    .with_location(source_span_to_location(&span))
                    .with_note(format!("got {}×{} matrix", rows, cols)));
                }
                // Result is a vector with same number of components as matrix columns
                // For vec3 × mat3, result is vec3 (but conceptually row vector)
                // GLSL treats this as returning a column vector
                return Ok(Type::vector_type(&Type::Float, cols).unwrap());
            }

            // Matrix × Matrix: linear algebra multiplication
            if lhs_ty.is_matrix() && rhs_ty.is_matrix() {
                let (lhs_rows, lhs_cols) = lhs_ty.matrix_dims().unwrap();
                let (rhs_rows, rhs_cols) = rhs_ty.matrix_dims().unwrap();
                
                if lhs_cols != rhs_rows {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        format!("matrix × matrix dimension mismatch: {}×{} × {}×{} requires {} == {}", 
                            lhs_rows, lhs_cols, rhs_rows, rhs_cols, lhs_cols, rhs_rows)
                    )
                    .with_location(source_span_to_location(&span)));
                }
                // Result is lhs_rows × rhs_cols matrix
                // For now, we only support square matrices, so result type matches lhs
                if lhs_rows == rhs_cols {
                    return Ok(lhs_ty.clone());
                }
                // Non-square not yet supported
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "non-square matrix multiplication not yet supported"
                )
                .with_location(source_span_to_location(&span)));
            }

            Err(GlslError::new(
                ErrorCode::E0106,
                "matrix multiplication requires matrix and scalar/vector/matrix operands"
            )
            .with_location(source_span_to_location(&span)))
        }

        Div => {
            // Matrix / Scalar: component-wise division
            if lhs_ty.is_matrix() && rhs_ty.is_scalar() {
                if !rhs_ty.is_numeric() {
                    return Err(GlslError::new(
                        ErrorCode::E0106,
                        "matrix / scalar requires numeric scalar"
                    )
                    .with_location(source_span_to_location(&span)));
                }
                return Ok(lhs_ty.clone());
            }
            Err(GlslError::new(
                ErrorCode::E0106,
                "matrix division only supports matrix / scalar"
            )
            .with_location(source_span_to_location(&span)))
        }

        _ => Err(GlslError::new(
            ErrorCode::E0106,
            format!("operator {:?} not supported for matrices", op)
        )
        .with_location(source_span_to_location(&span))),
    }
}

/// Parse swizzle string and return the number of components
/// Validates that the swizzle is valid for the given vector size
fn parse_swizzle_length(swizzle: &str, max_components: usize) -> Result<usize, GlslError> {
    if swizzle.is_empty() {
        return Err(GlslError::new(ErrorCode::E0113, "empty swizzle"));
    }
    
    if swizzle.len() > 4 {
        return Err(GlslError::new(
            ErrorCode::E0113,
            format!("swizzle can have at most 4 components, got {}", swizzle.len())
        ));
    }
    
    // Determine naming set and validate consistency
    let mut xyzw_count = 0;
    let mut rgba_count = 0;
    let mut stpq_count = 0;
    
    for ch in swizzle.chars() {
        match ch {
            'x' | 'y' | 'z' | 'w' => xyzw_count += 1,
            'r' | 'g' | 'b' | 'a' => rgba_count += 1,
            's' | 't' | 'p' | 'q' => stpq_count += 1,
            _ => return Err(GlslError::new(ErrorCode::E0113, format!("invalid swizzle character: '{}'", ch))),
        }
    }
    
    let sets_used = (xyzw_count > 0) as u32 + (rgba_count > 0) as u32 + (stpq_count > 0) as u32;
    if sets_used > 1 {
        return Err(GlslError::new(
            ErrorCode::E0113,
            format!("swizzle '{}' mixes component naming sets (xyzw/rgba/stpq)", swizzle)
        ));
    }
    
    // Validate each component is within bounds
    let naming_set = if xyzw_count > 0 {
        ('x', 'y', 'z', 'w')
    } else if rgba_count > 0 {
        ('r', 'g', 'b', 'a')
    } else {
        ('s', 't', 'p', 'q')
    };
    
    for ch in swizzle.chars() {
        let idx = match ch {
            'x' | 'r' | 's' => 0,
            'y' | 'g' | 't' => 1,
            'z' | 'b' | 'p' => 2,
            'w' | 'a' | 'q' => 3,
            _ => return Err(GlslError::new(ErrorCode::E0113, format!("invalid component '{}'", ch))),
        };
        
        if idx >= max_components {
            return Err(GlslError::new(
                ErrorCode::E0111,
                format!("component '{}' not valid for vector with {} components", ch, max_components)
            ));
        }
    }
    
    Ok(swizzle.len())
}

