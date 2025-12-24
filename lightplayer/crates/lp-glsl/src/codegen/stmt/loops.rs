use glsl::syntax::IterationStatement;

use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};

/// Emit iteration statement (dispatch to specific loop types)
pub fn emit_iteration_stmt(
    ctx: &mut CodegenContext,
    iteration: &IterationStatement,
) -> Result<(), GlslError> {
    use glsl::syntax::IterationStatement;

    match iteration {
        IterationStatement::While(condition, body) => {
            crate::codegen::stmt::loop_while::emit_loop_while_stmt(ctx, condition, body)
        }
        IterationStatement::DoWhile(body, expr) => {
            crate::codegen::stmt::loop_do_while::emit_loop_do_while_stmt(ctx, body, expr)
        }
        IterationStatement::For(init, rest, body) => {
            crate::codegen::stmt::loop_for::emit_loop_for_stmt(ctx, init, rest, body)
        }
    }
}

/// Shared helper: translate condition expression to boolean value
pub fn translate_condition(
    ctx: &mut CodegenContext,
    condition: &glsl::syntax::Condition,
) -> Result<cranelift_codegen::ir::Value, GlslError> {
    match condition {
        glsl::syntax::Condition::Expr(expr) => {
            let (vals, ty) = ctx.translate_expr_typed(expr)?;
            // Validate that condition is bool type (GLSL spec requirement)
            crate::semantic::type_check::check_condition(&ty)?;
            // Condition must be scalar, so we take the first (and only) value
            Ok(vals.into_iter().next().ok_or_else(|| {
                GlslError::new(ErrorCode::E0400, "condition expression produced no value")
            })?)
        }
        _ => Err(GlslError::new(
            ErrorCode::E0400,
            "only expression conditions supported",
        )),
    }
}
