//! Type conversion instruction converters (fcvt_from_sint, fcvt_from_uint).

use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

use super::transform::WalkCommand;
use super::types::{float_to_fixed16x16, float_to_fixed32x32};

/// Convert FcvtFromSint to fixed-point constant
/// This converts an integer to F32, which we then convert to fixed-point
pub(super) fn convert_fcvt_from_sint(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    // Check if result is F32
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    // Get the integer input (should already be mapped through value_map)
    let arg = if let InstructionData::Unary { opcode: _, arg } = inst_data {
        *value_map.get(arg).unwrap_or(arg)
    } else {
        return WalkCommand::Continue;
    };

    // Convert integer to fixed-point
    // fcvt_from_sint converts int -> f32
    // We need to convert int -> fixed-point
    // Fixed-point representation: value = int_value << shift_amount
    // So we scale the integer by the shift amount

    let mut cursor = FuncCursor::new(func).at_inst(inst);
    let int_type = cursor.func.dfg.value_type(arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Create conversion sequence before the current instruction
    // Scale the integer: int_value << shift_amount
    let new_result = if int_type == target_type {
        // Same type - just scale it
        let shift_const = cursor.ins().iconst(int_type, shift_amount as i64);
        cursor.ins().ishl(arg, shift_const)
    } else if int_type == cranelift_codegen::ir::types::I32
        && target_type == cranelift_codegen::ir::types::I64
    {
        // i32 -> i64: extend first, then shift
        let extended = cursor.ins().sextend(target_type, arg);
        let shift_const = cursor.ins().iconst(target_type, shift_amount as i64);
        cursor.ins().ishl(extended, shift_const)
    } else {
        // Unsupported conversion
        return WalkCommand::Continue;
    };

    // Detach old results and replace instruction with placeholder
    // The actual result is in new_result and mapped in value_map
    cursor.func.dfg.detach_inst_results(inst);
    let target_type = format.cranelift_type();
    cursor.func.dfg.replace(inst).iconst(target_type, 0);

    // Add to value_map
    value_map.insert(old_result, new_result);

    WalkCommand::Continue
}

/// Convert FcvtFromUint to fixed-point constant
pub(super) fn convert_fcvt_from_uint(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    // Similar to convert_fcvt_from_sint but for unsigned integers
    let inst_data = &func.dfg.insts[inst];

    // Check if result is F32
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    // Get the integer input (should already be mapped through value_map)
    let arg = if let InstructionData::Unary { opcode: _, arg } = inst_data {
        *value_map.get(arg).unwrap_or(arg)
    } else {
        return WalkCommand::Continue;
    };

    // Convert unsigned integer to fixed-point
    // Similar to convert_fcvt_from_sint but use unsigned extension
    let mut cursor = FuncCursor::new(func).at_inst(inst);
    let int_type = cursor.func.dfg.value_type(arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Create conversion sequence before the current instruction
    let new_result = if int_type == target_type {
        let shift_const = cursor.ins().iconst(int_type, shift_amount as i64);
        cursor.ins().ishl(arg, shift_const)
    } else if int_type == cranelift_codegen::ir::types::I32
        && target_type == cranelift_codegen::ir::types::I64
    {
        // i32 -> i64: extend first (unsigned), then shift
        let extended = cursor.ins().uextend(target_type, arg);
        let shift_const = cursor.ins().iconst(target_type, shift_amount as i64);
        cursor.ins().ishl(extended, shift_const)
    } else {
        return WalkCommand::Continue;
    };

    // Detach old results and replace instruction with placeholder
    // The actual result is in new_result and mapped in value_map
    cursor.func.dfg.detach_inst_results(inst);
    let target_type = format.cranelift_type();
    cursor.func.dfg.replace(inst).iconst(target_type, 0);

    // Add to value_map
    value_map.insert(old_result, new_result);

    WalkCommand::Continue
}
