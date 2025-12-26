use glsl::syntax::Expr;

use crate::frontend::codegen::context::CodegenContext;
use crate::error::GlslError;

/// Emit expression statement (expression followed by semicolon)
pub fn emit_expr_stmt(ctx: &mut CodegenContext, expr: &Expr) -> Result<(), GlslError> {
    ctx.translate_expr(expr)?;
    Ok(())
}
