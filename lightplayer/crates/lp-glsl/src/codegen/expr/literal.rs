use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::error::GlslError;
use glsl::syntax::Expr;
use cranelift_codegen::ir::{types, Value, InstBuilder};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_literal(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    match expr {
        Expr::IntConst(n, _) => {
            let val = ctx.builder.ins().iconst(types::I32, *n as i64);
            Ok((vec![val], GlslType::Int))
        }
        Expr::FloatConst(f, _) => {
            let val = ctx.builder.ins().f32const(*f);
            Ok((vec![val], GlslType::Float))
        }
        Expr::BoolConst(b, _) => {
            let val = ctx.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
            Ok((vec![val], GlslType::Bool))
        }
        _ => unreachable!("translate_literal called on non-literal"),
    }
}

