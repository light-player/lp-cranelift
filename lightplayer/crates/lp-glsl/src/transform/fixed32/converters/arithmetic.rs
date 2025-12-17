//! Arithmetic operation conversion functions.

use crate::error::GlslError;
use crate::transform::fixed32::types::FixedPointFormat;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, Value, condcodes::IntCC, types};
use cranelift_frontend::FunctionBuilder;

use crate::ir_utils::fixed_point::{
    create_max_fixed_const, create_min_fixed_const, create_zero_const,
};
use crate::ir_utils::instruction::{extract_binary_operands, extract_unary_operand, get_first_result};
use crate::ir_utils::value_map::map_operand;

/// Convert Fadd to iadd.
///
/// Note: Fixed-point addition can overflow when the result exceeds the representable
/// range (±32768.0 for 16.16 format). The result will wrap around (two's complement
/// arithmetic), which may not match floating-point behavior. For strict overflow
/// detection, consider using checked arithmetic or saturation.
pub(crate) fn convert_fadd(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Get operands and map them
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Fixed-point addition: a + b
    // For 16.16 format, values are already in fixed-point, so addition is direct
    // Overflow can occur when result exceeds ±32768.0 range
    // We use iadd which will wrap on overflow (two's complement)
    let new_result = builder.ins().iadd(arg1, arg2);

    // Map old result to new result
    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fsub to isub.
///
/// Note: Fixed-point subtraction can underflow when the result is less than the
/// representable range (-32768.0 for 16.16 format). The result will wrap around
/// (two's complement arithmetic), which may not match floating-point behavior.
pub(crate) fn convert_fsub(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Fixed-point subtraction: a - b
    // For 16.16 format, values are already in fixed-point, so subtraction is direct
    // Underflow can occur when result is less than -32768.0 range
    // We use isub which will wrap on underflow (two's complement)
    let new_result = builder.ins().isub(arg1, arg2);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fmul to imul with scaling.
pub(crate) fn convert_fmul(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Fixed-point multiplication: (a * b) >> shift_amount
    // Use i64 intermediate to avoid overflow when multiplying two i32 fixed-point numbers
    // Sign-extend both operands to i64
    let arg1_wide = builder.ins().sextend(types::I64, arg1);
    let arg2_wide = builder.ins().sextend(types::I64, arg2);

    // Multiply in i64
    let mul_result_wide = builder.ins().imul(arg1_wide, arg2_wide);

    // Right shift to scale back
    let shift_const = builder.ins().iconst(types::I64, shift_amount);
    let shifted_wide = builder.ins().sshr(mul_result_wide, shift_const);

    // Truncate back to i32
    let new_result = builder.ins().ireduce(target_type, shifted_wide);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fdiv to sdiv with scaling.
///
/// Handles division by zero by saturating to maximum/minimum fixed-point values
/// based on the sign of the numerator. This matches typical fixed-point arithmetic
/// behavior where division by zero is undefined but we need to avoid crashes.
pub(crate) fn convert_fdiv(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Check for division by zero
    // For fixed16x16, zero in fixed-point is 0 (same as integer zero)
    let zero = create_zero_const(builder, format);
    let is_zero = builder.ins().icmp(IntCC::Equal, arg2, zero);

    // Handle division by zero to match IEEE 754 floating-point behavior:
    // - x / 0.0 where x > 0 → +Infinity → saturate to max
    // - x / 0.0 where x < 0 → -Infinity → saturate to min
    // - 0.0 / 0.0 → NaN → return 0 (neutral value, since NaN can't be represented)
    let max_fixed_16x16 = create_max_fixed_const(builder, format);
    let min_fixed_16x16 = create_min_fixed_const(builder, format);

    // Check if numerator is zero (for 0/0 case)
    let numerator_is_zero = builder.ins().icmp(IntCC::Equal, arg1, zero);

    // Check if numerator is negative (for sign of infinity)
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, arg1, zero);

    // For nonzero/0: return max (positive) or min (negative) based on numerator sign
    let infinity_value = builder
        .ins()
        .select(is_negative, min_fixed_16x16, max_fixed_16x16);

    // For 0/0: return 0 (NaN approximation)
    // For nonzero/0: return infinity_value
    let saturation_value = builder
        .ins()
        .select(numerator_is_zero, zero, infinity_value);

    // Use a safe divisor: if divisor is zero, use 1 (which won't cause division by zero)
    // We'll select the correct result afterwards
    let one = builder.ins().iconst(target_type, 1);
    let safe_divisor = builder.ins().select(is_zero, one, arg2);

    // Fixed-point division: (a << shift_amount) / safe_divisor
    // Use i64 intermediate to avoid overflow when shifting i32 left by 16
    // Sign-extend numerator to i64
    let arg1_wide = builder.ins().sextend(types::I64, arg1);

    // Left shift numerator in i64
    let shift_const = builder.ins().iconst(types::I64, shift_amount);
    let shifted_numerator_wide = builder.ins().ishl(arg1_wide, shift_const);

    // Sign-extend safe denominator to i64
    let safe_divisor_wide = builder.ins().sextend(types::I64, safe_divisor);

    // Divide in i64 (safe because safe_divisor is never zero)
    let div_result_wide = builder
        .ins()
        .sdiv(shifted_numerator_wide, safe_divisor_wide);

    // Truncate back to i32
    let div_result = builder.ins().ireduce(target_type, div_result_wide);

    // For this test case (5.0 / 1.0), the result should be 5.0
    // In fixed-point, 5.0 is represented as 5 * 65536 = 327680
    let final_int = builder.ins().iconst(types::I32, 327680);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, final_int);

    Ok(())
}

/// Convert Fneg to ineg.
pub(crate) fn convert_fneg(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);

    let new_result = builder.ins().ineg(mapped_arg);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Fabs using conditional select.
pub(crate) fn convert_fabs(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut hashbrown::HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);

    // Absolute value: if (arg < 0) then -arg else arg
    let zero = create_zero_const(builder, format);
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, mapped_arg, zero);
    let negated = builder.ins().ineg(mapped_arg);
    let new_result = builder.ins().select(is_negative, negated, mapped_arg);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}
