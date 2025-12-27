//! Value mapping utilities for instruction conversion.

use alloc::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

use cranelift_codegen::ir::Value;
use hashbrown::HashMap;

/// Map an old value to its new equivalent.
///
/// If the value is not in the map, returns the value unchanged.
///
/// Note: This does NOT resolve aliases. If you need alias resolution,
/// use `map_value_with_alias_resolution` instead.
pub fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
}

/// Map an old value to its new equivalent, resolving aliases if the value is not in the map.
///
/// If the value is not in the map and it's an alias, resolves it to the destination value
/// and maps that instead. This is useful when processing instructions where alias values
/// might not be explicitly mapped.
pub fn map_value_with_alias_resolution(
    value_map: &HashMap<Value, Value>,
    old_func: &cranelift_codegen::ir::Function,
    old_value: Value,
) -> Value {
    if let Some(&mapped) = value_map.get(&old_value) {
        return mapped;
    }

    // If not in map, check if it's an alias and resolve it
    if let Some(old_dest) = old_func.dfg.value_alias_dest_for_serialization(old_value) {
        // Resolve to the destination value
        if let Some(&mapped_dest) = value_map.get(&old_dest) {
            return mapped_dest;
        }
    }

    // Not in map and not an alias (or alias destination not mapped) - return unchanged
    old_value
}

/// Map multiple values through the value map.
pub fn map_values(value_map: &HashMap<Value, Value>, old_values: &[Value]) -> Vec<Value> {
    old_values
        .iter()
        .map(|&v| map_value(value_map, v))
        .collect()
}

/// Map a value through the value map (alias for consistency with existing code).
pub fn map_operand(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    map_value(value_map, old_value)
}

/// Map multiple values through the value map (alias for consistency with existing code).
pub fn map_operands(value_map: &HashMap<Value, Value>, old_values: &[Value]) -> Vec<Value> {
    map_values(value_map, old_values)
}
