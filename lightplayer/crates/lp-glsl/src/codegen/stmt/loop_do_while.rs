use glsl::syntax::{Expr, Statement};

use crate::codegen::context::CodegenContext;
use crate::error::GlslError;
use cranelift_codegen::ir::InstBuilder;

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
    ctx.builder.ins().jump(body_block, &[]);

    // Body
    ctx.builder.switch_to_block(body_block);
    ctx.builder.seal_block(body_block);
    ctx.emit_statement(body)?;
    ctx.builder.ins().jump(header_block, &[]);

    // Header: evaluate condition
    ctx.builder.switch_to_block(header_block);
    let condition_value = ctx.translate_expr(condition)?;
    ctx.builder
        .ins()
        .brif(condition_value, body_block, &[], exit_block, &[]);

    // Exit
    ctx.builder.switch_to_block(exit_block);
    ctx.builder.seal_block(header_block);
    ctx.builder.seal_block(exit_block);

    ctx.loop_stack.pop();

    Ok(())
}
