//! Common helper functions for instruction conversion.
//!
//! This module provides shared utilities used across different converter modules
//! to reduce code duplication and ensure consistent error handling.

use crate::error::{ErrorCode, GlslError};
use crate::transform::fixed32::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value, types};
use cranelift_frontend::FunctionBuilder;

// Define map_value locally to match the HashMap type used by converters
// Converters use hashbrown::HashMap, so we match that
#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as HashMap;
#[cfg(feature = "std")]
use hashbrown::HashMap;

pub(crate) fn map_value(value_map: &HashMap<Value, Value>, old_value: Value) -> Value {
    *value_map.get(&old_value).unwrap_or(&old_value)
}

/// Extract binary operands from an instruction.
///
/// Returns an error if the instruction is not in Binary format.
pub fn extract_binary_operands(
    old_func: &Function,
    old_inst: Inst,
) -> Result<(Value, Value), GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Binary { args, .. } = inst_data {
        Ok((args[0], args[1]))
    } else {
        Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Expected binary instruction format, got: {:?} (opcode: {:?})",
                inst_data,
                old_func.dfg.insts[old_inst].opcode()
            ),
        ))
    }
}

/// Extract unary operand from an instruction.
///
/// Returns an error if the instruction is not in Unary format.
pub fn extract_unary_operand(old_func: &Function, old_inst: Inst) -> Result<Value, GlslError> {
    let inst_data = &old_func.dfg.insts[old_inst];
    if let InstructionData::Unary { arg, .. } = inst_data {
        Ok(*arg)
    } else {
        Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Expected unary instruction format, got: {:?} (opcode: {:?})",
                inst_data,
                old_func.dfg.insts[old_inst].opcode()
            ),
        ))
    }
}

/// Map a value through the value map, returning the mapped value.
///
/// This is a convenience wrapper that accepts hashbrown::HashMap.
pub fn map_operand(value_map: &hashbrown::HashMap<Value, Value>, old_value: Value) -> Value {
    // Convert to our internal HashMap type
    use hashbrown::HashMap as StdHashMap;
    // We need to work with the same type, so just call map_value directly
    // Since both are HashMap<Value, Value>, we can use as_ref pattern
    *value_map.get(&old_value).unwrap_or(&old_value)
}

/// Map multiple values through the value map.
pub fn map_operands(
    value_map: &hashbrown::HashMap<Value, Value>,
    old_values: &[Value],
) -> Vec<Value> {
    old_values
        .iter()
        .map(|&v| map_operand(value_map, v))
        .collect()
}

/// Get the first result value from an instruction.
pub fn get_first_result(old_func: &Function, old_inst: Inst) -> Value {
    old_func.dfg.first_result(old_inst)
}

/// Verify that a value has the expected type after conversion.
///
/// Returns an error if the type doesn't match, providing context about
/// the instruction and expected vs actual types.
pub fn verify_converted_type(
    builder: &FunctionBuilder,
    value: Value,
    expected_type: types::Type,
    old_func: &Function,
    old_inst: Inst,
) -> Result<(), GlslError> {
    let actual_type = builder.func.dfg.value_type(value);
    if actual_type != expected_type {
        return Err(GlslError::new(
            ErrorCode::E0301,
            format!(
                "Type mismatch after conversion: instruction {:?} (opcode: {:?}) expected type {:?}, got {:?}",
                old_inst,
                old_func.dfg.insts[old_inst].opcode(),
                expected_type,
                actual_type
            ),
        ));
    }
    Ok(())
}

/// Create an error for an unexpected instruction format.
///
/// Provides detailed context including the instruction, opcode, and actual format.
pub fn unexpected_format_error(
    old_func: &Function,
    old_inst: Inst,
    expected_format: &str,
) -> GlslError {
    let inst_data = &old_func.dfg.insts[old_inst];
    let opcode = inst_data.opcode();
    GlslError::new(
        ErrorCode::E0301,
        format!(
            "{} instruction has unexpected format: expected {}, got {:?} (opcode: {:?})",
            opcode, expected_format, inst_data, opcode
        ),
    )
}

/// Get the maximum representable value for a fixed-point format.
pub fn max_fixed_value(format: FixedPointFormat) -> i32 {
    match format {
        FixedPointFormat::Fixed16x16 => 0x7FFF0000i32, // 32767.0 in 16.16 format
        FixedPointFormat::Fixed32x32 => i32::MAX,      // Not fully implemented
    }
}

/// Get the minimum representable value for a fixed-point format.
pub fn min_fixed_value(format: FixedPointFormat) -> i32 {
    match format {
        FixedPointFormat::Fixed16x16 => 0x80000000u32 as i32, // -32768.0 in 16.16 format (i32::MIN)
        FixedPointFormat::Fixed32x32 => i32::MIN,             // Not fully implemented
    }
}

/// Create a constant for the maximum fixed-point value.
pub fn create_max_fixed_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    let max_val = max_fixed_value(format);
    builder.ins().iconst(target_type, max_val as i64)
}

/// Create a constant for the minimum fixed-point value.
pub fn create_min_fixed_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    let min_val = min_fixed_value(format);
    builder.ins().iconst(target_type, min_val as i64)
}

/// Create a zero constant for the target fixed-point type.
pub fn create_zero_const(builder: &mut FunctionBuilder, format: FixedPointFormat) -> Value {
    let target_type = format.cranelift_type();
    builder.ins().iconst(target_type, 0)
}
