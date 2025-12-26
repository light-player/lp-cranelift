use glsl::syntax::{Condition, Statement};

use crate::frontend::codegen::context::CodegenContext;
use crate::frontend::codegen::stmt::loops::translate_condition;
use crate::error::GlslError;

/// Emit while loop statement
pub fn emit_loop_while_stmt(
    ctx: &mut CodegenContext,
    condition: &Condition,
    body: &Statement,
) -> Result<(), GlslError> {
    let header_block = ctx.builder.create_block();
    let body_block = ctx.builder.create_block();
    let exit_block = ctx.builder.create_block();

    // Push loop context for break/continue
    ctx.loop_stack.push(crate::frontend::codegen::context::LoopContext {
        continue_target: header_block,
        exit_block,
    });

    // Jump to header
    ctx.emit_branch(header_block)?;

    // Header: evaluate condition
    // Don't seal header yet - it will receive a back edge from body
    ctx.switch_to_block(header_block);
    let condition_value = translate_condition(ctx, condition)?;
    ctx.emit_cond_branch(condition_value, body_block, exit_block)?;

    // Body
    ctx.emit_block(body_block);
    ctx.enter_scope(); // Enter scope for body variables
    ctx.emit_statement(body)?;
    ctx.exit_scope(); // Exit scope for body variables
    ctx.emit_branch(header_block)?; // Loop back

    // Now seal header block - all predecessors (initial jump + back edge) are known
    ctx.seal_block(header_block);

    // Exit - seal immediately since all predecessors are known
    ctx.emit_block(exit_block);

    // Pop loop context
    ctx.loop_stack.pop();

    Ok(())
}
