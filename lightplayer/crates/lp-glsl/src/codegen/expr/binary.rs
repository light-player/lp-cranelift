use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::{infer_binary_result_type, promote_numeric};
use crate::error::{ErrorCode, GlslError};
use glsl::syntax::Expr;
use cranelift_codegen::ir::{types, Value, condcodes::{IntCC, FloatCC}, InstBuilder};

use super::vector;
use super::matrix;
use super::coercion;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_binary(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let Expr::Binary(op, lhs, rhs, span) = expr else {
        unreachable!("translate_binary called on non-binary expr");
    };
    
    let (lhs_vals, lhs_ty) = ctx.translate_expr_typed(lhs)?;
    let (rhs_vals, rhs_ty) = ctx.translate_expr_typed(rhs)?;

    // Delegate to matrix/vector/scalar handlers
    if lhs_ty.is_matrix() || rhs_ty.is_matrix() {
        matrix::translate_matrix_binary(ctx, op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, span.clone())
    } else if lhs_ty.is_vector() || rhs_ty.is_vector() {
        vector::translate_vector_binary(ctx, op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, Some(span.clone()))
    } else {
        translate_scalar_binary(ctx, op, lhs_vals[0], &lhs_ty, rhs_vals[0], &rhs_ty, span.clone())
    }
}

fn translate_scalar_binary(
    ctx: &mut CodegenContext,
    op: &glsl::syntax::BinaryOp,
    lhs_val: Value,
    lhs_ty: &GlslType,
    rhs_val: Value,
    rhs_ty: &GlslType,
    span: glsl::syntax::SourceSpan,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    use glsl::syntax::BinaryOp::*;

    // Infer result type and validate
    let result_ty = infer_binary_result_type(op, lhs_ty, rhs_ty, span.clone())?;

    // Check if operator is logical or comparison (skip promotion for these)
    let is_logical = matches!(op, And | Or | Xor);
    let is_comparison = matches!(op, Equal | NonEqual | LT | GT | LTE | GTE);

    let (lhs_val, rhs_val, operand_ty) = if is_logical {
        // Logical operators: both operands must be Bool (validated above)
        // Skip promotion - use Bool directly
        (lhs_val, rhs_val, GlslType::Bool)
    } else if is_comparison {
        // Comparison operators: operands are numeric, may need promotion
        // if different types (Int vs Float)
        let common_ty = promote_numeric(lhs_ty, rhs_ty);
        let lhs_val = coercion::coerce_to_type(ctx, lhs_val, lhs_ty, &common_ty)?;
        let rhs_val = coercion::coerce_to_type(ctx, rhs_val, rhs_ty, &common_ty)?;
        (lhs_val, rhs_val, common_ty)
    } else {
        // Arithmetic operators: promote to common type
        let common_ty = promote_numeric(lhs_ty, rhs_ty);
        let lhs_val = coercion::coerce_to_type(ctx, lhs_val, lhs_ty, &common_ty)?;
        let rhs_val = coercion::coerce_to_type(ctx, rhs_val, rhs_ty, &common_ty)?;
        (lhs_val, rhs_val, common_ty)
    };

    // Generate operation
    let result_val = translate_scalar_binary_op(ctx, op, lhs_val, rhs_val, &operand_ty)?;
    Ok((vec![result_val], result_ty))
}

// Internal function for scalar binary operations (used by vector/matrix modules)
pub fn translate_scalar_binary_op_internal(
    ctx: &mut CodegenContext,
    op: &glsl::syntax::BinaryOp,
    lhs: Value,
    rhs: Value,
    operand_ty: &GlslType,
) -> Result<Value, GlslError> {
    translate_scalar_binary_op(ctx, op, lhs, rhs, operand_ty)
}

fn translate_scalar_binary_op(
    ctx: &mut CodegenContext,
    op: &glsl::syntax::BinaryOp,
    lhs: Value,
    rhs: Value,
    operand_ty: &GlslType,
) -> Result<Value, GlslError> {
    use glsl::syntax::BinaryOp::*;

    let val = match op {
        // Arithmetic operators - dispatch based on type
        Add => match operand_ty {
            GlslType::Int => ctx.builder.ins().iadd(lhs, rhs),
            GlslType::Float => ctx.builder.ins().fadd(lhs, rhs),
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("add not supported for {:?}", operand_ty))),
        },
        Sub => match operand_ty {
            GlslType::Int => ctx.builder.ins().isub(lhs, rhs),
            GlslType::Float => ctx.builder.ins().fsub(lhs, rhs),
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("sub not supported for {:?}", operand_ty))),
        },
        Mult => match operand_ty {
            GlslType::Int => ctx.builder.ins().imul(lhs, rhs),
            GlslType::Float => ctx.builder.ins().fmul(lhs, rhs),
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("mult not supported for {:?}", operand_ty))),
        },
        Div => match operand_ty {
            GlslType::Int => ctx.builder.ins().sdiv(lhs, rhs),
            GlslType::Float => ctx.builder.ins().fdiv(lhs, rhs),
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("div not supported for {:?}", operand_ty))),
        },

        // Comparison operators - dispatch based on type
        // icmp/fcmp return I1, but GLSL bool is I8, so convert
        Equal => {
            let cmp_result = match operand_ty {
                GlslType::Int => ctx.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                GlslType::Float => ctx.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("equal not supported for {:?}", operand_ty))),
            };
            // Convert I1 to I8: select 1 if true, 0 if false
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().select(cmp_result, one, zero)
        },
        NonEqual => {
            let cmp_result = match operand_ty {
                GlslType::Int => ctx.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                GlslType::Float => ctx.builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("nonEqual not supported for {:?}", operand_ty))),
            };
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().select(cmp_result, one, zero)
        },
        LT => {
            let cmp_result = match operand_ty {
                GlslType::Int => ctx.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
                GlslType::Float => ctx.builder.ins().fcmp(FloatCC::LessThan, lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("LT not supported for {:?}", operand_ty))),
            };
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().select(cmp_result, one, zero)
        },
        GT => {
            let cmp_result = match operand_ty {
                GlslType::Int => ctx.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                GlslType::Float => ctx.builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("GT not supported for {:?}", operand_ty))),
            };
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().select(cmp_result, one, zero)
        },
        LTE => {
            let cmp_result = match operand_ty {
                GlslType::Int => ctx.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                GlslType::Float => ctx.builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("LTE not supported for {:?}", operand_ty))),
            };
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().select(cmp_result, one, zero)
        },
        GTE => {
            let cmp_result = match operand_ty {
                GlslType::Int => ctx.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
                GlslType::Float => ctx.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("GTE not supported for {:?}", operand_ty))),
            };
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().select(cmp_result, one, zero)
        },

        // Logical operators (bool only, already validated by type_check)
        And => {
            // Logical AND: both operands must be bool (I8)
            // Result: (lhs != 0) && (rhs != 0) ? 1 : 0
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let lhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, lhs, zero);
            let rhs_nonzero = ctx.builder.ins().icmp(IntCC::NotEqual, rhs, zero);
            // Result is 1 if both are non-zero, 0 otherwise
            let rhs_result = ctx.builder.ins().select(rhs_nonzero, one, zero);
            ctx.builder.ins().select(lhs_nonzero, rhs_result, zero)
        }
        Or | Xor => {
            return Err(GlslError::new(ErrorCode::E0400, format!("logical operator {:?} not yet implemented", op)));
        }

        _ => return Err(GlslError::new(ErrorCode::E0400, format!("binary operator not supported yet: {:?}", op))),
    };

    Ok(val)
}

