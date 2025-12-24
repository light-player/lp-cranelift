use glsl::syntax::{Expr, Statement};

use crate::codegen::context::CodegenContext;
use crate::error::GlslError;

/// Emit do-while loop statement
pub fn emit_loop_do_while_stmt(
    ctx: &mut CodegenContext,
    body: &Statement,
    condition: &Expr,
) -> Result<(), GlslError> {
    // Create blocks (don't seal yet)
    let body_block = ctx.builder.create_block();
    let cond_block = ctx.builder.create_block();
    let exit_block = ctx.builder.create_block();

    ctx.loop_stack.push(crate::codegen::context::LoopContext {
        continue_target: cond_block,
        exit_block,
    });

    // Branch directly to body (do-while always executes once)
    ctx.emit_branch(body_block)?;

    // Body: switch to but don't seal yet - will receive back edge from condition block
    ctx.switch_to_block(body_block);
    ctx.emit_statement(body)?;
    ctx.emit_branch(cond_block)?;

    // Condition: switch to but don't seal yet - will receive back edge from body
    ctx.switch_to_block(cond_block);
    let condition_value = ctx.translate_expr(condition)?;
    ctx.emit_cond_branch(condition_value, body_block, exit_block)?;
    // Now body_block is declared as successor - safe to seal both

    // Seal blocks now that all predecessors are known
    ctx.seal_block(body_block);
    ctx.seal_block(cond_block);

    // Exit - seal immediately since all predecessors are known
    ctx.emit_block(exit_block);

    ctx.loop_stack.pop();

    Ok(())
}
