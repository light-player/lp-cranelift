//! Type inference and validation for GLSL expressions
//! Implements GLSL spec type rules for Phase 3

use crate::semantic::types::Type;
use crate::semantic::scope::SymbolTable;
use glsl::syntax::{Expr, BinaryOp, UnaryOp};

#[cfg(feature = "std")]
use std::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

/// Infer the result type of an expression
pub fn infer_expr_type(
    expr: &Expr,
    symbols: &SymbolTable,
) -> Result<Type, String> {
    match expr {
        Expr::IntConst(_) => Ok(Type::Int),
        Expr::FloatConst(_) => Ok(Type::Float),
        Expr::BoolConst(_) => Ok(Type::Bool),
        Expr::DoubleConst(_) => Ok(Type::Float), // Treat as float for now

        Expr::Variable(ident) => {
            let var = symbols.lookup_variable(&ident.0)
                .ok_or_else(|| format!("Undefined variable: {}", ident.0))?;
            Ok(var.ty.clone())
        }

        Expr::Binary(op, lhs, rhs) => {
            let lhs_ty = infer_expr_type(lhs, symbols)?;
            let rhs_ty = infer_expr_type(rhs, symbols)?;
            infer_binary_result_type(op, &lhs_ty, &rhs_ty)
        }

        Expr::Unary(op, expr) => {
            let expr_ty = infer_expr_type(expr, symbols)?;
            infer_unary_result_type(op, &expr_ty)
        }

        Expr::Assignment(lhs, _op, _rhs) => {
            // Assignment result has same type as LHS
            infer_expr_type(lhs, symbols)
        }

        _ => Err(format!("Cannot infer type for: {:?}", expr)),
    }
}

/// Infer result type of binary operation (with implicit conversion)
/// Implements GLSL spec: operators.adoc:775-855
pub fn infer_binary_result_type(
    op: &BinaryOp,
    lhs_ty: &Type,
    rhs_ty: &Type,
) -> Result<Type, String> {
    use BinaryOp::*;

    match op {
        // Arithmetic operators: operands must be numeric
        Add | Sub | Mult | Div => {
            if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
                return Err(format!(
                    "Arithmetic operator {:?} requires numeric operands, got {:?} and {:?}",
                    op, lhs_ty, rhs_ty
                ));
            }
            // Result type is the promoted type
            Ok(promote_numeric(lhs_ty, rhs_ty))
        }

        // Comparison operators: operands must be compatible, result is bool
        Equal | NonEqual | LT | GT | LTE | GTE => {
            if !lhs_ty.is_numeric() || !rhs_ty.is_numeric() {
                return Err(format!(
                    "Comparison operator {:?} requires numeric operands, got {:?} and {:?}",
                    op, lhs_ty, rhs_ty
                ));
            }
            Ok(Type::Bool)
        }

        // Logical operators: must be bool
        And | Or | Xor => {
            if lhs_ty != &Type::Bool || rhs_ty != &Type::Bool {
                return Err(format!(
                    "Logical operator {:?} requires bool operands, got {:?} and {:?}",
                    op, lhs_ty, rhs_ty
                ));
            }
            Ok(Type::Bool)
        }

        _ => Err(format!("Unsupported binary operator: {:?}", op)),
    }
}

/// Infer result type of unary operation
pub fn infer_unary_result_type(
    op: &UnaryOp,
    operand_ty: &Type,
) -> Result<Type, String> {
    use UnaryOp::*;

    match op {
        Minus => {
            if !operand_ty.is_numeric() {
                return Err(format!(
                    "Unary minus requires numeric operand, got {:?}",
                    operand_ty
                ));
            }
            Ok(operand_ty.clone())
        }

        Not => {
            if operand_ty != &Type::Bool {
                return Err(format!(
                    "Logical NOT requires bool operand, got {:?}",
                    operand_ty
                ));
            }
            Ok(Type::Bool)
        }

        _ => Err(format!("Unsupported unary operator: {:?}", op)),
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
    from == to || matches!((from, to), (Type::Int, Type::Float))
}

/// Validate assignment types
pub fn check_assignment(lhs_ty: &Type, rhs_ty: &Type) -> Result<(), String> {
    if !can_implicitly_convert(rhs_ty, lhs_ty) {
        return Err(format!(
            "Type mismatch: cannot assign {:?} to {:?}",
            rhs_ty, lhs_ty
        ));
    }
    Ok(())
}

/// Validate condition expression type (must be bool)
pub fn check_condition(cond_ty: &Type) -> Result<(), String> {
    if cond_ty != &Type::Bool {
        return Err(format!(
            "Condition must be bool type, got {:?}",
            cond_ty
        ));
    }
    Ok(())
}

/// Check vector constructor arguments and infer result type
pub fn check_vector_constructor(
    type_name: &str,
    args: &[Type],
) -> Result<Type, String> {
    let result_type = parse_vector_type_name(type_name)?;
    let component_count = result_type.component_count()
        .ok_or_else(|| format!("{} is not a vector type", type_name))?;
    let base_type = result_type.vector_base_type().unwrap();

    // Case 1: Single scalar - broadcast to all components
    if args.len() == 1 && args[0].is_scalar() {
        // Check implicit conversion is allowed
        if !can_implicitly_convert(&args[0], &base_type) {
            return Err(format!(
                "Cannot construct {} from {:?}",
                type_name, args[0]
            ));
        }
        return Ok(result_type);
    }

    // Case 2: Single vector - type conversion
    if args.len() == 1 && args[0].is_vector() {
        if args[0].component_count() != Some(component_count) {
            return Err(format!(
                "Cannot construct {} from {:?} (component count mismatch)",
                type_name, args[0]
            ));
        }
        // Check base type conversion is allowed
        let src_base = args[0].vector_base_type().unwrap();
        if !can_implicitly_convert(&src_base, &base_type) {
            return Err(format!(
                "Cannot construct {} from {:?}",
                type_name, args[0]
            ));
        }
        return Ok(result_type);
    }

    // Case 3: Multiple arguments - concatenation
    let total_components = count_total_components(args)?;
    if total_components != component_count {
        return Err(format!(
            "{} constructor requires {} components, got {}",
            type_name, component_count, total_components
        ));
    }

    // Validate each argument can convert to base type
    for arg in args {
        let arg_base = if arg.is_vector() {
            arg.vector_base_type().unwrap()
        } else {
            arg.clone()
        };
        
        if !can_implicitly_convert(&arg_base, &base_type) {
            return Err(format!(
                "Cannot use {:?} in {} constructor",
                arg, type_name
            ));
        }
    }

    Ok(result_type)
}

fn parse_vector_type_name(name: &str) -> Result<Type, String> {
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
        _ => Err(format!("Unknown type: {}", name)),
    }
}

fn count_total_components(args: &[Type]) -> Result<usize, String> {
    let mut total = 0;
    for arg in args {
        if let Some(count) = arg.component_count() {
            total += count;
        } else if arg.is_scalar() {
            total += 1;
        } else {
            return Err(format!("Invalid constructor argument: {:?}", arg));
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

