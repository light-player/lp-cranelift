//! Math function conversion functions.

use crate::error::GlslError;
use crate::backend::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, Value, types};
use cranelift_frontend::FunctionBuilder;

use crate::backend::ir_utils::fixed_point::create_zero_const;
use crate::backend::ir_utils::instruction::{extract_unary_operand, get_first_result};
use crate::backend::ir_utils::value_map::map_operand;

/// Convert Ceil instruction.
pub(crate) fn convert_ceil(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Ceil: round up to nearest integer
    // In fixed-point: (value + (1 << shift) - 1) >> shift, then << shift
    let mask = (1i64 << shift_amount) - 1;
    let mask_const = builder.ins().iconst(target_type, mask);
    let added = builder.ins().iadd(mapped_arg, mask_const);
    let shift_const = builder.ins().iconst(target_type, shift_amount);
    let rounded = builder.ins().sshr(added, shift_const);
    let new_result = builder.ins().ishl(rounded, shift_const);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Floor instruction.
pub(crate) fn convert_floor(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Floor: round down to nearest integer
    // In fixed-point: value >> shift, then << shift
    let shift_const = builder.ins().iconst(target_type, shift_amount);
    let rounded = builder.ins().sshr(mapped_arg, shift_const);
    let new_result = builder.ins().ishl(rounded, shift_const);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Sqrt instruction using unrolled Newton-Raphson.
///
/// For fixed16x16 format:
/// - Input: x_fixed (i32, represents x * 65536)
/// - Output: sqrt(x) in fixed-point format (sqrt(x) * 65536)
/// - Algorithm: We compute sqrt(x_fixed << 16) which equals sqrt(x) * 65536
///   This gives us the result directly in fixed-point format.
pub(crate) fn convert_sqrt(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
    _block_map: &hashbrown::HashMap<
        cranelift_codegen::ir::Block,
        cranelift_codegen::ir::Block,
    >,
) -> Result<(), GlslError> {
    use cranelift_codegen::ir::condcodes::IntCC;

    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Handle edge cases with select (no branching)
    let zero = create_zero_const(builder, format);
    let is_zero = builder.ins().icmp(IntCC::Equal, mapped_arg, zero);
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, mapped_arg, zero);
    let is_invalid = builder.ins().bor(is_zero, is_negative);

    // Convert to i64 for intermediate calculations
    let x_fixed_wide = builder.ins().sextend(types::I64, mapped_arg);

    // Scale up for better precision: x_scaled = x_fixed << 16
    // This allows us to compute sqrt(x_fixed << 16) = sqrt(x) * 65536 directly
    let shift_16 = builder.ins().iconst(types::I64, shift_amount);
    let x_scaled = builder.ins().ishl(x_fixed_wide, shift_16);

    // Initial guess: max(x_scaled >> 16, 1) to avoid division by zero
    // This is a rough approximation: sqrt(x_scaled) ≈ x_scaled >> 16 for large values
    let shift_16_for_guess = builder.ins().iconst(types::I64, shift_amount);
    let guess0_wide = builder.ins().sshr(x_scaled, shift_16_for_guess);
    let one_wide = builder.ins().iconst(types::I64, 1);
    let guess0_min = builder.ins().smax(guess0_wide, one_wide);

    // Newton-Raphson iteration 1: guess1 = (guess0 + x_scaled / guess0) >> 1
    let div1 = builder.ins().sdiv(x_scaled, guess0_min);
    let sum1 = builder.ins().iadd(guess0_min, div1);
    let shift_1 = builder.ins().iconst(types::I64, 1);
    let guess1_wide = builder.ins().sshr(sum1, shift_1);

    // Newton-Raphson iteration 2
    let div2 = builder.ins().sdiv(x_scaled, guess1_wide);
    let sum2 = builder.ins().iadd(guess1_wide, div2);
    let guess2_wide = builder.ins().sshr(sum2, shift_1);

    // Newton-Raphson iteration 3
    let div3 = builder.ins().sdiv(x_scaled, guess2_wide);
    let sum3 = builder.ins().iadd(guess2_wide, div3);
    let guess3_wide = builder.ins().sshr(sum3, shift_1);

    // Newton-Raphson iteration 4 (for better precision)
    let div4 = builder.ins().sdiv(x_scaled, guess3_wide);
    let sum4 = builder.ins().iadd(guess3_wide, div4);
    let guess4_wide = builder.ins().sshr(sum4, shift_1);

    // guess4_wide = sqrt(x_scaled) = sqrt(x_fixed << 16) = sqrt(x) * 65536
    // This is already in fixed-point format, so we just truncate to i32
    let result = builder.ins().ireduce(target_type, guess4_wide);

    // Handle edge cases: if input was zero or negative, return 0
    let new_result = builder.ins().select(is_invalid, zero, result);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}
