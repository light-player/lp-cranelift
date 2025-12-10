//! Math function converters (sqrt, ceil, floor).

use crate::error::GlslError;
use crate::transform::fixed_point::types::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::collections::BTreeMap as ValueMap;
#[cfg(feature = "std")]
use std::collections::HashMap as ValueMap;

use cranelift_codegen::cursor::{Cursor, FuncCursor};
use cranelift_codegen::ir::{
    Function, Inst, InstBuilder, InstructionData, Value, condcodes::IntCC,
};

/// Convert Sqrt to integer square root using Newton's method
pub(super) fn convert_sqrt(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        let arg = *value_map.get(arg).unwrap_or(arg);
        let old_result = func.dfg.first_result(inst);
        let target_type = format.cranelift_type();

        let mut cursor = FuncCursor::new(func).at_inst(inst);

        // Handle zero case
        let zero = cursor.ins().iconst(target_type, 0);
        let is_zero = cursor.ins().icmp(IntCC::Equal, arg, zero);

        // Use Newton's method for integer square root
        // Start with initial guess: x >> (shift/2) as a rough approximation
        let shift_amount = format.shift_amount();
        let initial_shift = shift_amount / 2;

        // Initial guess: x >> (shift/2)
        let shift_const = cursor.ins().iconst(target_type, initial_shift);
        let x_guess = cursor.ins().sshr(arg, shift_const);

        // Newton's method: x_new = (x_old + n/x_old) >> 1
        // We need to iterate a few times. For simplicity, let's do 5 iterations.
        // Extend to larger type for intermediate calculations
        match format {
            FixedPointFormat::Fixed16x16 => {
                // Extend to I64 for calculations
                let x_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg);
                let guess_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I64, x_guess);

                // Iterate: new_guess = (old_guess + (x << shift) / old_guess) >> 1
                let x_scaled = cursor.ins().ishl(x_ext, shift_const);
                let mut current_guess = guess_ext;

                // 5 iterations of Newton's method
                for _ in 0..5 {
                    // div = x_scaled / current_guess
                    let div = cursor.ins().sdiv(x_scaled, current_guess);
                    // sum = current_guess + div
                    let sum = cursor.ins().iadd(current_guess, div);
                    // new_guess = sum >> 1
                    let one = cursor.ins().iconst(cranelift_codegen::ir::types::I64, 1);
                    current_guess = cursor.ins().sshr(sum, one);
                }

                // Convert back to I32
                let result_64 = current_guess;
                let result_32 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, result_64);

                // Handle zero case
                let new_result = cursor.ins().select(is_zero, zero, result_32);
                value_map.insert(old_result, new_result);
            }
            FixedPointFormat::Fixed32x32 => {
                // Extend to I128 for calculations
                let x_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I128, arg);
                let guess_ext = cursor
                    .ins()
                    .sextend(cranelift_codegen::ir::types::I128, x_guess);

                let x_scaled = cursor.ins().ishl(x_ext, shift_const);
                let mut current_guess = guess_ext;

                // 7 iterations for better precision with 32.32 format
                for _ in 0..7 {
                    let div = cursor.ins().sdiv(x_scaled, current_guess);
                    let sum = cursor.ins().iadd(current_guess, div);
                    let one = cursor.ins().iconst(cranelift_codegen::ir::types::I64, 1);
                    current_guess = cursor.ins().sshr(sum, one);
                }

                let result_128 = current_guess;
                let result_64 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I64, result_128);

                let new_result = cursor.ins().select(is_zero, zero, result_64);
                value_map.insert(old_result, new_result);
            }
        }

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Ceil to fixed-point ceiling operation
pub(super) fn convert_ceil(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        let arg = *value_map.get(arg).unwrap_or(arg);
        let old_result = func.dfg.first_result(inst);
        let shift_amount = format.shift_amount();
        let target_type = format.cranelift_type();

        let mut cursor = FuncCursor::new(func).at_inst(inst);

        // For fixed-point ceiling: ceil(x) means round up the fractional part
        // If x has any fractional bits, add 1 to the integer part
        match format {
            FixedPointFormat::Fixed16x16 => {
                let shift_const = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, shift_amount);
                let arg_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg);

                // mask = (1 << shift) - 1
                let mask = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, (1 << shift_amount) - 1);

                // fractional_part = x & mask
                let frac = cursor.ins().band(arg_ext, mask);
                let has_frac = cursor.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    frac,
                    0,
                );

                // int_part = x >> shift
                let int_part = cursor.ins().sshr(arg_ext, shift_const);

                // int_part_plus_one = int_part + (has_frac ? 1 : 0)
                let one_inc = cursor.ins().iconst(cranelift_codegen::ir::types::I64, 1);
                let int_plus_one = cursor.ins().iadd(int_part, one_inc);
                let rounded_int = cursor.ins().select(has_frac, int_plus_one, int_part);

                // result = rounded_int << shift
                let result_64 = cursor.ins().ishl(rounded_int, shift_const);
                let result_32 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, result_64);

                value_map.insert(old_result, result_32);
            }
            FixedPointFormat::Fixed32x32 => {
                let shift_const = cursor
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I64, shift_amount);

                let mask_minus_one = cursor
                    .ins()
                    .iconst(target_type, ((1_u64 << shift_amount) - 1) as i64);

                let frac = cursor.ins().band(arg, mask_minus_one);
                let has_frac = cursor.ins().icmp_imm(
                    cranelift_codegen::ir::condcodes::IntCC::NotEqual,
                    frac,
                    0,
                );

                let int_part = cursor.ins().sshr(arg, shift_const);
                let one_inc = cursor.ins().iconst(target_type, 1);
                let int_plus_one = cursor.ins().iadd(int_part, one_inc);
                let rounded_int = cursor.ins().select(has_frac, int_plus_one, int_part);

                let new_result = cursor.ins().ishl(rounded_int, shift_const);
                value_map.insert(old_result, new_result);
            }
        }

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}

/// Convert Floor to fixed-point floor operation
pub(super) fn convert_floor(
    func: &mut Function,
    inst: Inst,
    format: FixedPointFormat,
    value_map: &mut ValueMap<Value, Value>,
) -> Result<(), GlslError> {
    let inst_data = &func.dfg.insts[inst];
    if let InstructionData::Unary { opcode: _, arg } = inst_data {
        let arg = *value_map.get(arg).unwrap_or(arg);
        let old_result = func.dfg.first_result(inst);
        let shift_amount = format.shift_amount();

        let mut cursor = FuncCursor::new(func).at_inst(inst);

        // For fixed-point floor: floor(x) = (x >> shift) << shift
        // This truncates the fractional bits

        let shift_const = cursor
            .ins()
            .iconst(cranelift_codegen::ir::types::I64, shift_amount);

        match format {
            FixedPointFormat::Fixed16x16 => {
                let arg_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, arg);
                let int_part = cursor.ins().sshr(arg_ext, shift_const);
                let result_64 = cursor.ins().ishl(int_part, shift_const);
                let result_32 = cursor
                    .ins()
                    .ireduce(cranelift_codegen::ir::types::I32, result_64);
                value_map.insert(old_result, result_32);
            }
            FixedPointFormat::Fixed32x32 => {
                let int_part = cursor.ins().sshr(arg, shift_const);
                let new_result = cursor.ins().ishl(int_part, shift_const);
                value_map.insert(old_result, new_result);
            }
        }

        // Detach and remove old instruction
        cursor.func.dfg.detach_inst_results(inst);
        cursor.goto_inst(inst);
        cursor.remove_inst();
    }

    Ok(())
}
