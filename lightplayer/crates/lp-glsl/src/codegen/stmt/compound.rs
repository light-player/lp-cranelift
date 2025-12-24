use glsl::syntax::CompoundStatement;

use crate::codegen::context::CodegenContext;
use crate::error::GlslError;

/// Emit compound statement (block of statements in { ... })
pub fn emit_compound_stmt(
    ctx: &mut CodegenContext,
    compound: &CompoundStatement,
) -> Result<(), GlslError> {
    // Translate all statements in the compound block
    for stmt in &compound.statement_list {
        ctx.emit_statement(stmt)?;
    }
    Ok(())
}
