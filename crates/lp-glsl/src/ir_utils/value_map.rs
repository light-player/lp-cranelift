//! Value mapping utilities for instruction conversion.

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::Value;
use hashbrown::HashMap;

/// Map an old value to its new equivalent.
///
/// If the value is not in the map, returns the value unchanged.
pub fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
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
