//! Block creation and parameter management for fixed-point transformation.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{Block, Function, Value, types};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Create all blocks in the new function and map block parameters.
pub(super) fn create_and_map_blocks(
    old_func: &Function,
    builder: &mut FunctionBuilder,
    block_map: &mut HashMap<Block, Block>,
    value_map: &mut HashMap<Value, Value>,
) -> Result<(), GlslError> {
    // Collect blocks first to avoid borrow conflicts
    let old_blocks: Vec<Block> = old_func.layout.blocks().collect();
    let old_entry_block = old_func.layout.entry_block();

    // Create blocks in same order as original
    for old_block in &old_blocks {
        let new_block = builder.create_block();
        block_map.insert(*old_block, new_block);
    }

    // Handle entry block specially - use function parameters
    if let Some(old_entry) = old_entry_block {
        let new_entry = block_map[&old_entry];

        // For entry block, use function parameters (this creates params matching the signature)
        builder.append_block_params_for_function_params(new_entry);

        // Map old entry block params to new entry block params
        let old_params = old_func.dfg.block_params(old_entry);
        let new_params = builder.block_params(new_entry);

        // Verify counts match
        if old_params.len() != new_params.len() {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "Entry block parameter count mismatch: old={}, new={}",
                    old_params.len(),
                    new_params.len()
                ),
            ));
        }

        // Map old params to new params
        for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
            value_map.insert(*old_param, *new_param);
        }
    }

    // Don't create block parameters for non-entry blocks here
    // They will be created on-demand when we encounter jumps/brifs that target them
    // This handles the case where parameters are added dynamically via append_block_param

    Ok(())
}

/// Map function parameters (verify entry block params match function signature).
pub(super) fn map_function_params(
    old_func: &Function,
    entry_block: Block,
    new_entry_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Function parameters are the block parameters of the entry block
    // They should already be mapped in create_and_map_blocks
    // But verify they're correct
    let old_params = old_func.dfg.block_params(entry_block);
    let new_params = builder.block_params(new_entry_block);

    // Verify counts match
    if old_params.len() != new_params.len() {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Function parameter count mismatch: old={}, new={}",
                old_params.len(),
                new_params.len()
            ),
        ));
    }

    // Verify types are correct
    let target_type = format.cranelift_type();
    for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
        let old_type = old_func.dfg.value_type(*old_param);
        let new_type = builder.func.dfg.value_type(*new_param);

        if old_type == types::F32 {
            if new_type != target_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "F32 function parameter not converted: expected {:?}, got {:?}",
                        target_type, new_type
                    ),
                ));
            }
        } else {
            if new_type != old_type {
                return Err(GlslError::new(
                    ErrorCode::E0301,
                    format!(
                        "Non-F32 function parameter type changed: old={:?}, new={:?}",
                        old_type, new_type
                    ),
                ));
            }
        }

        // Verify mapping exists
        if value_map.get(old_param) != Some(new_param) {
            return Err(GlslError::new(
                ErrorCode::E0301,
                format!(
                    "Function parameter not mapped: old_param={:?}, expected new_param={:?}",
                    old_param, new_param
                ),
            ));
        }
    }

    Ok(())
}

/// Ensure block parameters exist for a target block based on what the old block has.
/// This creates parameters on-demand when they're needed for jumps/brifs.
///
/// The key insight: we check what parameters the old block actually has at conversion time,
/// not just what the current instruction passes. This handles the case where parameters
/// are added dynamically via append_block_param.
pub(super) fn ensure_block_params(
    old_func: &Function,
    old_block: Block,
    new_block: Block,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
    _expected_args: &[Value], // Not used, but kept for API consistency
) -> Result<(), GlslError> {
    let target_type = format.cranelift_type();

    // Get current parameter count in new block
    let current_param_count = builder.func.dfg.num_block_params(new_block);

    // Get the old block's actual parameters (what it has at conversion time)
    // This is the source of truth - if the old block has N parameters, the new block should too
    let old_params = old_func.dfg.block_params(old_block);
    let expected_param_count = old_params.len();

    // If we need more parameters, add them based on the old block's parameters
    if expected_param_count > current_param_count {
        // Determine types for new parameters based on the old block's parameters
        let mut new_param_types = Vec::new();
        for &old_param in old_params.iter().skip(current_param_count) {
            let old_type = old_func.dfg.value_type(old_param);
            let new_type = if old_type == types::F32 {
                target_type
            } else {
                old_type
            };
            new_param_types.push(new_type);
        }

        // Add the missing parameters
        for &param_type in &new_param_types {
            builder.append_block_param(new_block, param_type);
        }

        // Map old params to new params (only the newly added ones)
        let new_params = builder.block_params(new_block);
        for i in current_param_count..expected_param_count {
            if i < old_params.len() && i < new_params.len() {
                value_map.insert(old_params[i], new_params[i]);
            }
        }
    }

    Ok(())
}
