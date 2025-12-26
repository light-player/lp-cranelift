use glsl::syntax::CompoundStatement;

use crate::frontend::codegen::context::CodegenContext;
use crate::error::GlslError;

/// Emit compound statement (block of statements in { ... })
pub fn emit_compound_stmt(
    ctx: &mut CodegenContext,
    compound: &CompoundStatement,
) -> Result<(), GlslError> {
    ctx.enter_scope(); // Enter scope for compound block
    // Translate all statements in the compound block
    for stmt in &compound.statement_list {
        ctx.emit_statement(stmt)?;
    }
    ctx.exit_scope(); // Exit scope for compound block
    Ok(())
}
