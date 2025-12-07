use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::infer_unary_result_type;
use crate::error::{ErrorCode, GlslError};
use glsl::syntax::Expr;
use cranelift_codegen::ir::{types, Value, condcodes::IntCC, InstBuilder};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_unary(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let Expr::Unary(op, operand, span) = expr else {
        unreachable!("translate_unary called on non-unary expr");
    };
    
    let (vals, ty) = ctx.translate_expr_typed(operand)?;
    
    if vals.len() != 1 {
        return Err(GlslError::new(ErrorCode::E0400, "vector unary ops not yet implemented"));
    }

    let val = vals[0];
    let result_ty = infer_unary_result_type(op, &ty, span.clone())?;
    let result_val = translate_unary_op(ctx, op, val, &ty)?;
    Ok((vec![result_val], result_ty))
}

fn translate_unary_op(
    ctx: &mut CodegenContext,
    op: &glsl::syntax::UnaryOp,
    val: Value,
    operand_ty: &GlslType,
) -> Result<Value, GlslError> {
    use glsl::syntax::UnaryOp::*;

    let result = match op {
        Minus => match operand_ty {
            GlslType::Int => ctx.builder.ins().ineg(val),
            GlslType::Float => ctx.builder.ins().fneg(val),
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("unary minus not supported for {:?}", operand_ty))),
        },
        Not => {
            if operand_ty != &GlslType::Bool {
                return Err(GlslError::new(ErrorCode::E0107, format!("logical NOT requires bool, got {:?}", operand_ty)));
            }
            let zero = ctx.builder.ins().iconst(types::I8, 0);
            ctx.builder.ins().icmp(IntCC::Equal, val, zero)
        }
        _ => return Err(GlslError::new(ErrorCode::E0400, format!("unary operator not supported yet: {:?}", op))),
    };

    Ok(result)
}

