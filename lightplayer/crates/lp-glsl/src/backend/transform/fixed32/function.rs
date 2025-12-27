//! Main function rewriting entry point for fixed-point transformation.
//!
//! This module implements a complete rewrite of functions using FunctionBuilder,
//! creating a new function from scratch rather than mutating in place.

use crate::backend::transform::fixed32::blocks::map_function_params;
use crate::backend::transform::fixed32::instructions::convert_all_instructions;
use crate::backend::transform::fixed32::signature::convert_signature;
use crate::backend::transform::fixed32::types::FixedPointFormat;
use crate::backend::util::clif_copy::{
    copy_stack_slots, copy_value_aliases, create_blocks, map_entry_block_params,
};
use crate::error::GlslError;

use alloc::{format, vec::Vec};

use cranelift_codegen::ir::Function;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext};
use hashbrown::HashMap;

/// Main entry point for rewriting a function.
///
/// Creates a new function with converted signature and converts all
/// instructions from F32 to fixed-point representation.
pub fn rewrite_function(
    old_func: &Function,
    format: FixedPointFormat,
) -> Result<Function, GlslError> {
    // 1. Convert signature
    let new_sig = convert_signature(&old_func.signature, format);

    // 2. Create new function
    let mut new_func = Function::with_name_signature(old_func.name.clone(), new_sig);

    // 3. Copy stack slots from old function to new function
    copy_stack_slots(old_func, &mut new_func)?;

    // 4. Create builder context
    let mut builder_ctx = FunctionBuilderContext::new();

    // 5. Create a single builder that we'll reuse throughout
    let mut builder = FunctionBuilder::new(&mut new_func, &mut builder_ctx);

    // 6. Create maps for blocks, values, function refs, and signature refs
    let mut block_map = HashMap::new();
    let mut value_map = HashMap::new();
    let mut ext_func_map = HashMap::new();
    let mut sig_map = HashMap::new();

    // 7. Build blocks and map parameters using shared utility
    create_blocks(old_func, &mut builder, &mut block_map, &mut value_map)?;

    // 8. Get entry block and verify function parameters
    let entry_block = old_func.layout.entry_block().ok_or_else(|| {
        GlslError::new(
            crate::error::ErrorCode::E0301,
            "Function has no entry block",
        )
    })?;
    let new_entry_block = block_map[&entry_block];

    // Verify entry block parameters are correctly mapped (basic check)
    map_entry_block_params(
        old_func,
        entry_block,
        new_entry_block,
        &mut builder,
        &value_map,
    )?;

    // Also verify types are correct for transform (this is transform-specific)
    map_function_params(
        old_func,
        entry_block,
        new_entry_block,
        &mut builder,
        &value_map,
        format,
    )?;

    // 9. Convert instructions (this will switch to blocks as needed)
    convert_all_instructions(
        old_func,
        &mut builder,
        &mut value_map,
        &mut ext_func_map,
        &mut sig_map,
        format,
        &block_map,
    )?;

    // 10. Seal all blocks now that all instructions are converted
    builder.seal_all_blocks();

    // 11. Finalize builder (this clears the builder context)
    builder.finalize();

    // 12. Copy value aliases from old function to new function using shared utility
    copy_value_aliases(old_func, &mut new_func, &value_map)?;

    // 13. Return new function (builder is dropped, so we can return new_func)
    Ok(new_func)
}
