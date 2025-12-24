use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};

/// Emit break statement
pub fn emit_break_stmt(ctx: &mut CodegenContext) -> Result<(), GlslError> {
    let loop_ctx = ctx
        .loop_stack
        .last()
        .ok_or_else(|| GlslError::new(ErrorCode::E0400, "break statement outside of loop"))?;

    ctx.emit_branch(loop_ctx.exit_block)?;

    // Create unreachable block for subsequent code
    let unreachable = ctx.builder.create_block();
    ctx.emit_block(unreachable);

    Ok(())
}
