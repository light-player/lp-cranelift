use glsl::syntax::{Condition, Statement};

use crate::codegen::context::CodegenContext;
use crate::codegen::stmt::loops::translate_condition;
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
    ctx.loop_stack.push(crate::codegen::context::LoopContext {
        continue_target: header_block,
        exit_block,
    });

    // Jump to header
    ctx.emit_branch(header_block)?;

    // Header: evaluate condition
    ctx.emit_block(header_block);
    let condition_value = translate_condition(ctx, condition)?;
    ctx.emit_cond_branch(condition_value, body_block, exit_block)?;

    // Body
    ctx.emit_block(body_block);
    ctx.emit_statement(body)?;
    ctx.emit_branch(header_block)?; // Loop back

    // Exit
    ctx.emit_block(exit_block);

    // Pop loop context
    ctx.loop_stack.pop();

    Ok(())
}
