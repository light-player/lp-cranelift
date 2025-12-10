//! Arithmetic operation converters (add, sub, mul, div, neg, abs).

use crate::error::GlslError;
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{Function, Inst, InstBuilder, InstructionData, Value};

/// Convert Fadd to Iadd (fixed-point addition is direct integer addition)
pub(super) fn convert_fadd(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let old_result = func.dfg.first_result(inst);

        // Create new iadd instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().iadd(arg1, arg2);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Fsub to Isub (fixed-point subtraction is direct integer subtraction)
pub(super) fn convert_fsub(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let old_result = func.dfg.first_result(inst);

        // Create new isub instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().isub(arg1, arg2);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Fmul to fixed-point multiplication sequence
/// For fixed-point multiply: result = (a * b) >> shift_amount
pub(super) fn convert_fmul(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let result = func.dfg.first_result(inst);
        let shift_amount = format.shift_amount();

        // Create a cursor positioned at this instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);

        match format {
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
                let result_32 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, shifted);

                // Add to value map
                value_map.insert(result, result_32);

                // Detach and remove the original instruction
                cursor.func.dfg.detach_inst_results(inst);
                cursor.goto_inst(inst);
                cursor.remove_inst();
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
                let result_64 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I64, shifted);

                // Add to value map
                value_map.insert(result, result_64);

                // Detach and remove the original instruction
                cursor.func.dfg.detach_inst_results(inst);
                cursor.goto_inst(inst);
                cursor.remove_inst();
            }
        }
    }

    Ok(())
}

/// Convert Fdiv to fixed-point division sequence
/// For fixed-point divide: result = (a << shift_amount) / b
pub(super) fn convert_fdiv(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Binary { opcode: _, args } = inst_data {
        let arg1 = *value_map.get(&args[0]).unwrap_or(&args[0]);
        let arg2 = *value_map.get(&args[1]).unwrap_or(&args[1]);
        let result = func.dfg.first_result(inst);
        let shift_amount = format.shift_amount();

        let mut cursor = FuncCursor::new(func).at_inst(inst);

        match format {
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
                let result_32 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, div_result);

                // Add to value map
                value_map.insert(result, result_32);

                cursor.func.dfg.detach_inst_results(inst);
                cursor.goto_inst(inst);
                cursor.remove_inst();
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
                let result_64 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I64, div_result);

                // Add to value map
                value_map.insert(result, result_64);

                cursor.func.dfg.detach_inst_results(inst);
                cursor.goto_inst(inst);
                cursor.remove_inst();
            }
        }
    }

    Ok(())
}

/// Convert Fneg to Ineg (fixed-point negation is direct integer negation)
pub(super) fn convert_fneg(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        let arg = *value_map.get(arg).unwrap_or(arg);
        let old_result = func.dfg.first_result(inst);

        // Create new ineg instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().ineg(arg);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Fabs to Iabs (fixed-point absolute value is integer absolute value)
pub(super) fn convert_fabs(
    func: &mut Function,
    inst: Inst,
    _format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        let arg = *value_map.get(arg).unwrap_or(arg);
        let old_result = func.dfg.first_result(inst);

        // Create new iabs instruction
        let mut cursor = FuncCursor::new(func).at_inst(inst);
        let new_result = cursor.ins().iabs(arg);

        // Add to value map
        value_map.insert(old_result, new_result);

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}
