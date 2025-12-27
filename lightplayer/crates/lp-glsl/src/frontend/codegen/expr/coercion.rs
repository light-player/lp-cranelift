use crate::error::{ErrorCode, GlslError, source_span_to_location};
use crate::frontend::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use cranelift_codegen::ir::{
    InstBuilder, Value,
    condcodes::{FloatCC, IntCC},
    types,
};

pub fn coerce_to_type(
    ctx: &mut CodegenContext,
    val: Value,
    from_ty: &GlslType,
    to_ty: &GlslType,
) -> Result<Value, GlslError> {
    coerce_to_type_with_location(ctx, val, from_ty, to_ty, None)
}

pub fn coerce_to_type_with_location(
    ctx: &mut CodegenContext,
    val: Value,
    from_ty: &GlslType,
    to_ty: &GlslType,
    span: Option<glsl::syntax::SourceSpan>,
) -> Result<Value, GlslError> {
    if from_ty == to_ty {
        return Ok(val);
    }

    match (from_ty, to_ty) {
        (GlslType::Int, GlslType::Float) => {
            // int → float: fcvt_from_sint
            Ok(ctx.builder.ins().fcvt_from_sint(types::F32, val))
        }
        // Boolean to numeric conversions
        (GlslType::Bool, GlslType::Int) => {
            // bool → int: false → 0, true → 1
            // val is i8 (0 or 1), extend to i32
            Ok(ctx.builder.ins().uextend(types::I32, val))
        }
        (GlslType::Bool, GlslType::Float) => {
            // bool → float: false → 0.0, true → 1.0
            // val is i8 (0 or 1), convert to i32 then to float
            let i32_val = ctx.builder.ins().uextend(types::I32, val);
            Ok(ctx.builder.ins().fcvt_from_sint(types::F32, i32_val))
        }
        // TODO: Add (GlslType::Bool, GlslType::UInt) when UInt type is added
        // Numeric to boolean conversions
        (GlslType::Int, GlslType::Bool) => {
            // int → bool: 0 → false, non-zero → true
            // val is i32, compare with 0, convert result to i8
            let zero = ctx.builder.ins().iconst(types::I32, 0);
            let cmp = ctx.builder.ins().icmp(IntCC::NotEqual, val, zero);
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero_i8 = ctx.builder.ins().iconst(types::I8, 0);
            Ok(ctx.builder.ins().select(cmp, one, zero_i8))
        }
        (GlslType::Float, GlslType::Bool) => {
            // float → bool: 0.0 → false, non-zero → true
            // val is f32, compare with 0.0, convert result to i8
            let zero = ctx.builder.ins().f32const(0.0);
            let cmp = ctx.builder.ins().fcmp(FloatCC::NotEqual, val, zero);
            let one = ctx.builder.ins().iconst(types::I8, 1);
            let zero_i8 = ctx.builder.ins().iconst(types::I8, 0);
            Ok(ctx.builder.ins().select(cmp, one, zero_i8))
        }
        // TODO: Add (GlslType::UInt, GlslType::Bool) when UInt type is added
        _ => {
            let error_msg = format!("cannot implicitly convert {:?} to {:?}", from_ty, to_ty);
            let mut error = GlslError::new(ErrorCode::E0103, error_msg);
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            Err(error)
        }
    }
}
