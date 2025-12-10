//! Arithmetic operation converters (add, sub, mul, div, neg, abs).

use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

use super::transform::WalkCommand;

/// Convert Fadd to Iadd (fixed-point addition is direct integer addition)
pub(super) fn convert_fadd(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    // Check if this instruction needs conversion
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    if let InstructionData::Binary { opcode: _, args } = inst_data {
        // Map operands through value_map FIRST
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);

        // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
        // We're changing the result type (F32 -> I32/I64), so we must detach first.
        func.dfg.detach_inst_results(inst);

        // Replace instruction in-place
        let new_result = func.dfg.replace(inst).iadd(arg1, arg2);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}

/// Convert Fsub to Isub (fixed-point subtraction is direct integer subtraction)
pub(super) fn convert_fsub(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    // Check if this instruction needs conversion
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    if let InstructionData::Binary { opcode: _, args } = inst_data {
        // Map operands through value_map FIRST
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);

        // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
        // We're changing the result type (F32 -> I32/I64), so we must detach first.
        func.dfg.detach_inst_results(inst);

        // Replace instruction in-place
        let new_result = func.dfg.replace(inst).isub(arg1, arg2);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}

/// Convert Fmul to fixed-point multiplication sequence
/// For fixed-point multiply: result = (a * b) >> shift_amount
pub(super) fn convert_fmul(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let old_result = func.dfg.first_result(inst);

    // Only convert if result type is F32
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue;
    }

    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        // Map operands through value_map FIRST
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let shift_amount = format.shift_amount();

        // Create a cursor positioned at this instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);

        let new_result = match format {
            FixedPointFormat::Fixed16x16 => {
                // For 16.16: result = (a * b) >> 16
                // Extend to 64-bit, multiply, shift, truncate
                let a_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I64, arg1);
                let b_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I64, arg2);
                let mul_64 = cursor.ins().imul(a_ext, b_ext);
                let shift_const_64 = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let shifted = cursor.ins().sshr(mul_64, shift_const_64);
                cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, shifted)
            }
            FixedPointFormat::Fixed32x32 => {
                // For 32.32: result = (a * b) >> 32
                // Use i128 arithmetic
                let a_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I128, arg1);
                let b_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I128, arg2);
                let mul_128 = cursor.ins().imul(a_ext, b_ext);
                let shift_const_128 = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let shifted = cursor.ins().sshr(mul_128, shift_const_128);
                cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I64, shifted)
            }
        };

        // Detach old instruction results
        cursor.func.dfg.detach_inst_results(inst);

        // Add to value_map immediately - this maps old F32 result to new I32/I64 result
        // All uses of old_result will be redirected to new_result via value_map during forward_walk
        // Note: We do NOT use change_to_alias here because it's designed for same-type aliasing,
        // and we're converting from F32 to I32/I64. The value_map mechanism handles cross-type
        // value replacement correctly.
        value_map.insert(old_result, new_result);

        // Replace old instruction with a harmless instruction (iconst 0)
        // The alias ensures correctness, this is just to clean up the instruction
        let target_type = format.cranelift_type();
        cursor.func.dfg.replace(inst).iconst(target_type, 0);
    }

    WalkCommand::Continue
}

/// Convert Fdiv to fixed-point division sequence
/// For fixed-point divide: result = (a << shift_amount) / b
pub(super) fn convert_fdiv(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let old_result = func.dfg.first_result(inst);

    // Only convert if result type is F32
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue;
    }

    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        // Map operands through value_map FIRST
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let shift_amount = format.shift_amount();

        let mut cursor = FuncCursor::new(func).at_inst(inst);

        let new_result = match format {
            FixedPointFormat::Fixed16x16 => {
                // For 16.16: result = (a << 16) / b
                // Extend to 64-bit to avoid overflow
                let a_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I64, arg1);
                let shift_const = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let a_shifted = cursor.ins().ishl(a_ext, shift_const);
                let b_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I64, arg2);
                let div_result = cursor.ins().sdiv(a_shifted, b_ext);
                cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, div_result)
            }
            FixedPointFormat::Fixed32x32 => {
                // For 32.32: result = (a << 32) / b
                // Extend to 128-bit
                let a_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I128, arg1);
                let shift_const = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let a_shifted = cursor.ins().ishl(a_ext, shift_const);
                let b_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I128, arg2);
                let div_result = cursor.ins().sdiv(a_shifted, b_ext);
                cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I64, div_result)
            }
        };

        // Detach old instruction results
        cursor.func.dfg.detach_inst_results(inst);

        // Add to value_map immediately - this maps old F32 result to new I32/I64 result
        // All uses of old_result will be redirected to new_result via value_map during forward_walk
        // Note: We do NOT use change_to_alias here because it's designed for same-type aliasing,
        // and we're converting from F32 to I32/I64. The value_map mechanism handles cross-type
        // value replacement correctly.
        value_map.insert(old_result, new_result);

        // Replace old instruction with a harmless instruction (iconst 0)
        // The alias ensures correctness, this is just to clean up the instruction
        let target_type = format.cranelift_type();
        cursor.func.dfg.replace(inst).iconst(target_type, 0);
    }

    WalkCommand::Continue
}

/// Convert Fneg to Ineg (fixed-point negation is direct integer negation)
pub(super) fn convert_fneg(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    // Check if this instruction needs conversion
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        // Map operand through value_map FIRST
        let arg = *value_map.get(arg).unwrap_or(arg);

        // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
        // We're changing the result type (F32 -> I32/I64), so we must detach first.
        func.dfg.detach_inst_results(inst);

        // Replace instruction in-place
        let new_result = func.dfg.replace(inst).ineg(arg);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}

/// Convert Fabs to Iabs (fixed-point absolute value is integer absolute value)
pub(super) fn convert_fabs(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> WalkCommand {
    let inst_data = &func.dfg.insts[inst];

    // Check if this instruction needs conversion
    let old_result = func.dfg.first_result(inst);
    if func.dfg.value_type(old_result) != cranelift_codegen::ir::types::F32 {
        return WalkCommand::Continue; // Not F32, skip
    }

    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        // Map operand through value_map FIRST
        let arg = *value_map.get(arg).unwrap_or(arg);

        // CRITICAL: Detach old results FIRST, otherwise replace() preserves them
        // We're changing the result type (F32 -> I32/I64), so we must detach first.
        func.dfg.detach_inst_results(inst);

        // Replace instruction in-place
        let new_result = func.dfg.replace(inst).iabs(arg);

        // Add to value_map immediately
        value_map.insert(old_result, new_result);
    }

    WalkCommand::Continue
}
