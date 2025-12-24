use crate::codegen::context::CodegenContext;
use crate::codegen::rvalue::RValue;
use crate::semantic::types::Type as GlslType;
use crate::error::GlslError;
use glsl::syntax::Expr;
use cranelift_codegen::ir::{types, InstBuilder};

/// Emit code to compute a literal as an RValue
pub fn emit_literal_rvalue(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<RValue, GlslError> {
    match expr {
        Expr::IntConst(n, _) => {
            let val = ctx.builder.ins().iconst(types::I32, *n as i64);
            Ok(RValue::from_scalar(val, GlslType::Int))
        }
        Expr::FloatConst(f, _) => {
            let val = ctx.builder.ins().f32const(*f);
            Ok(RValue::from_scalar(val, GlslType::Float))
        }
        Expr::BoolConst(b, _) => {
            let val = ctx.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
            Ok(RValue::from_scalar(val, GlslType::Bool))
        }
        _ => unreachable!("emit_literal_rvalue called on non-literal"),
    }
}

/// Legacy function for backwards compatibility
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_literal(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<cranelift_codegen::ir::Value>, GlslType), GlslError> {
    let rvalue = emit_literal_rvalue(ctx, expr)?;
    let ty = rvalue.ty().clone();
    Ok((rvalue.into_values(), ty))
}

