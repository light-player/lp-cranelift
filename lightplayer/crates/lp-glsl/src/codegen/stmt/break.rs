use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::InstBuilder;

/// Emit break statement
pub fn emit_break_stmt(ctx: &mut CodegenContext) -> Result<(), GlslError> {
    let loop_ctx = ctx
        .loop_stack
        .last()
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, "break statement outside of loop"))?;

    ctx.builder.ins().jump(loop_ctx.exit_block, &[]);

    // Create unreachable block for subsequent code
    let unreachable = ctx.builder.create_block();
    ctx.builder.switch_to_block(unreachable);
    ctx.builder.seal_block(unreachable);

    Ok(())
}
