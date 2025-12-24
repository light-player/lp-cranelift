use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::InstBuilder;

/// Emit continue statement
pub fn emit_continue_stmt(ctx: &mut CodegenContext) -> Result<(), GlslError> {
    let loop_ctx = ctx.loop_stack.last().ok_or_else(|| {
        GlslError::new(ErrorCode::E0400, "continue statement outside of loop")
    })?;

    ctx.builder.ins().jump(loop_ctx.continue_target, &[]);

    // Create unreachable block for subsequent code
    let unreachable = ctx.builder.create_block();
    ctx.builder.switch_to_block(unreachable);
    ctx.builder.seal_block(unreachable);

    Ok(())
}
