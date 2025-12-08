use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::error::{ErrorCode, GlslError, source_span_to_location};
use cranelift_codegen::ir::{types, Value, InstBuilder};

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
        _ => {
            let error_msg = format!("cannot implicitly convert {:?} to {:?}", from_ty, to_ty);
            let mut error = GlslError::new(ErrorCode::E0103, error_msg);
            if let Some(s) = span {
                error = error.with_location(source_span_to_location(&s));
            }
            Err(error)
        },
    }
}

