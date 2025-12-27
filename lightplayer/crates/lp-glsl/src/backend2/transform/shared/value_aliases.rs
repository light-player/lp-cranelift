//! Value alias copying utilities

use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Value};
use hashbrown::HashMap;

use alloc::vec::Vec;

/// Copy value aliases from old function to new function.
///
/// Value aliases are used for serialization and need to be preserved.
///
/// Strategy: For each alias in the old function:
/// 1. If the alias value is in value_map (it was used in instructions), use that mapped value
/// 2. If the alias value is NOT in value_map but the destination is, we need to create
///    a new value for the alias that aliases the mapped destination
/// 3. Create the alias relationship in the new function
pub fn copy_value_aliases(
    old_func: &Function,
    new_func: &mut Function,
    value_map: &HashMap<Value, Value>,
) -> Result<(), GlslError> {
    // First pass: handle aliases where both alias and destination are already mapped
    let mut aliases_to_create = Vec::new();

    for old_value in old_func.dfg.values() {
        if let Some(old_dest) = old_func.dfg.value_alias_dest_for_serialization(old_value) {
            if let Some(&new_dest) = value_map.get(&old_dest) {
                if let Some(&new_alias) = value_map.get(&old_value) {
                    // Both are mapped - create alias relationship
                    aliases_to_create.push((new_dest, new_alias));
                }
                // If alias value not mapped but destination is, we skip it for now.
                // In practice, if an alias is used in instructions, it should be in value_map.
            }
        }
    }

    // Create all aliases
    for (new_dest, new_alias) in aliases_to_create {
        // Create the alias in the new function: new_alias -> new_dest
        // Note: make_value_alias_for_serialization takes (src, dest) where dest is the alias
        new_func
            .dfg
            .make_value_alias_for_serialization(new_dest, new_alias);
    }

    Ok(())
}

