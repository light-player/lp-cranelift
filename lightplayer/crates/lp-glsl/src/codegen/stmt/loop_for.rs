use glsl::syntax::{ForInitStatement, ForRestStatement, Statement};

use crate::codegen::context::CodegenContext;
use crate::codegen::stmt::loops::translate_condition;
use crate::error::GlslError;
use cranelift_codegen::ir::InstBuilder;

/// Emit for loop statement
pub fn emit_loop_for_stmt(
    ctx: &mut CodegenContext,
    init: &ForInitStatement,
    rest: &ForRestStatement,
    body: &Statement,
) -> Result<(), GlslError> {
    // Translate init
    match init {
        glsl::syntax::ForInitStatement::Expression(Some(expr)) => {
            ctx.translate_expr(expr)?;
        }
        glsl::syntax::ForInitStatement::Declaration(decl) => {
            ctx.emit_declaration(decl)?;
        }
        glsl::syntax::ForInitStatement::Expression(None) => {
            // Empty init
        }
    }

    // Create blocks: header, body, update (for continue), exit
    let header_block = ctx.builder.create_block();
    let body_block = ctx.builder.create_block();
    let update_block = ctx.builder.create_block();
    let exit_block = ctx.builder.create_block();

    // For loops: continue should jump to update block, not header
    ctx.loop_stack.push(crate::codegen::context::LoopContext {
        continue_target: update_block,
        exit_block,
    });

    ctx.builder.ins().jump(header_block, &[]);

    // Header: evaluate condition
    ctx.builder.switch_to_block(header_block);
    let condition_value = if let Some(condition) = &rest.condition {
        translate_condition(ctx, condition)?
    } else {
        // No condition means infinite loop (while(true))
        ctx.builder
            .ins()
            .iconst(cranelift_codegen::ir::types::I8, 1)
    };
    ctx.builder
        .ins()
        .brif(condition_value, body_block, &[], exit_block, &[]);

    // Body
    ctx.builder.switch_to_block(body_block);
    ctx.builder.seal_block(body_block);
    ctx.emit_statement(body)?;
    ctx.builder.ins().jump(update_block, &[]);

    // Update block
    ctx.builder.switch_to_block(update_block);
    ctx.builder.seal_block(update_block);
    if let Some(update_expr) = &rest.post_expr {
        ctx.translate_expr(update_expr)?;
    }
    ctx.builder.ins().jump(header_block, &[]);

    // Exit
    ctx.builder.switch_to_block(exit_block);
    ctx.builder.seal_block(header_block);
    ctx.builder.seal_block(exit_block);

    ctx.loop_stack.pop();

    Ok(())
}
