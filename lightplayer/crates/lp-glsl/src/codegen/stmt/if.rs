use glsl::syntax::SelectionStatement;

use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::InstBuilder;

/// Emit if statement (selection statement)
pub fn emit_if_stmt(
    ctx: &mut CodegenContext,
    selection: &SelectionStatement,
) -> Result<(), GlslError> {
    use crate::error::{extract_span_from_expr, source_span_to_location};
    use glsl::syntax::SelectionRestStatement;

    // Translate condition and validate type
    let (cond_vals, cond_ty) = ctx.translate_expr_typed(&selection.cond)?;
    // Validate that condition is bool type (GLSL spec requirement)
    if cond_ty != crate::semantic::types::Type::Bool {
        let span = extract_span_from_expr(&selection.cond);
        let error = GlslError::new(ErrorCode::E0107, "condition must be bool type")
            .with_location(source_span_to_location(&span))
            .with_note(format!(
                "condition has type `{:?}`, expected `Bool`",
                cond_ty
            ));
        return Err(ctx.add_span_to_error(error, &span));
    }
    // Condition must be scalar, so we take the first (and only) value
    let condition_value = cond_vals.into_iter().next().ok_or_else(|| {
        GlslError::new(ErrorCode::E0400, "condition expression produced no value")
    })?;

    let then_block = ctx.builder.create_block();
    let merge_block = ctx.builder.create_block();

    match &selection.rest {
        SelectionRestStatement::Statement(then_stmt) => {
            // No else: branch to then or merge
            ctx.builder
                .ins()
                .brif(condition_value, then_block, &[], merge_block, &[]);

            // Then branch
            ctx.builder.switch_to_block(then_block);
            ctx.builder.seal_block(then_block);
            ctx.emit_statement(then_stmt)?;
            ctx.builder.ins().jump(merge_block, &[]);

            // Merge
            ctx.builder.switch_to_block(merge_block);
            ctx.builder.seal_block(merge_block);
        }
        SelectionRestStatement::Else(then_stmt, else_stmt) => {
            let else_block = ctx.builder.create_block();

            // Branch to then or else
            ctx.builder
                .ins()
                .brif(condition_value, then_block, &[], else_block, &[]);

            // Then branch
            ctx.builder.switch_to_block(then_block);
            ctx.builder.seal_block(then_block);
            ctx.emit_statement(then_stmt)?;
            ctx.builder.ins().jump(merge_block, &[]);

            // Else branch
            ctx.builder.switch_to_block(else_block);
            ctx.builder.seal_block(else_block);
            ctx.emit_statement(else_stmt)?;
            ctx.builder.ins().jump(merge_block, &[]);

            // Merge
            ctx.builder.switch_to_block(merge_block);
            ctx.builder.seal_block(merge_block);
        }
    }

    Ok(())
}
