use glsl::syntax::{Expr, Statement};

use crate::codegen::context::CodegenContext;
use crate::error::GlslError;

/// Emit do-while loop statement
pub fn emit_loop_do_while_stmt(
    ctx: &mut CodegenContext,
    body: &Statement,
    condition: &Expr,
) -> Result<(), GlslError> {
    let body_block = ctx.builder.create_block();
    let header_block = ctx.builder.create_block();
    let exit_block = ctx.builder.create_block();

    ctx.loop_stack.push(crate::codegen::context::LoopContext {
        continue_target: header_block,
        exit_block,
    });

    // Jump directly to body (do-while always executes once)
    ctx.emit_branch(body_block)?;

    // Body
    ctx.emit_block(body_block);
    ctx.emit_statement(body)?;
    ctx.emit_branch(header_block)?;

    // Header: evaluate condition
    // Don't seal header yet - it will receive a back edge from body
    ctx.switch_to_block(header_block);
    let condition_value = ctx.translate_expr(condition)?;
    ctx.emit_cond_branch(condition_value, body_block, exit_block)?;

    // Now seal header block - all predecessors (initial jump + back edge from body) are known
    ctx.seal_block(header_block);

    // Exit - seal immediately since all predecessors are known
    ctx.emit_block(exit_block);

    ctx.loop_stack.pop();

    Ok(())
}
