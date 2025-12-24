use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::error::{GlslError, extract_span_from_identifier, source_span_to_location};
use glsl::syntax::Expr;
use cranelift_codegen::ir::Value;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

pub fn translate_variable(
    ctx: &mut CodegenContext,
    expr: &Expr,
) -> Result<(Vec<Value>, GlslType), GlslError> {
    let Expr::Variable(ident, _span) = expr else {
        unreachable!("translate_variable called on non-variable");
    };
    
    let span = extract_span_from_identifier(ident);
    let vars = ctx
        .lookup_variables(&ident.name)
        .ok_or_else(|| {
            let error = GlslError::undefined_variable(&ident.name)
                .with_location(source_span_to_location(&span));
            ctx.add_span_to_error(error, &span)
        })?
        .to_vec(); // Clone to avoid borrow issues
    
    let ty = ctx
        .lookup_variable_type(&ident.name)
        .ok_or_else(|| {
            let error = GlslError::new(crate::error::ErrorCode::E0400, format!("variable type not found for `{}` during codegen", ident.name))
                .with_location(source_span_to_location(&span));
            ctx.add_span_to_error(error, &span)
        })?
        .clone();
    
    // Ensure we're in the correct block before reading variables
    // This is important when reading variables in merge blocks after control flow
    let _current_block = ctx.builder.current_block().expect("must be in a block to read variables");
    ctx.builder.ensure_inserted_block();
    
    // Read all component values fresh in the current block context
    // This ensures we get the correct SSA values for the current block
    let vals: Vec<Value> = vars.iter()
        .map(|&v| {
            // Force a fresh read of the variable in the current block
            ctx.builder.use_var(v)
        })
        .collect();
    
    Ok((vals, ty))
}





