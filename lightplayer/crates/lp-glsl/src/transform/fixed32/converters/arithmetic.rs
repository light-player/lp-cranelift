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

    // Use a safe divisor: if divisor is zero, use 1 (which won't cause division by zero)
    // We'll correct the result afterwards
    let one = builder.ins().iconst(target_type, 1);
    let is_zero_i32 = builder.ins().uextend(types::I32, is_zero);
    let not_is_zero_i32 = builder.ins().isub(one, is_zero_i32);
    let zero_contrib_safe = builder.ins().imul(is_zero_i32, one);
    let arg2_contrib = builder.ins().imul(not_is_zero_i32, arg2);
    let safe_divisor = builder.ins().iadd(zero_contrib_safe, arg2_contrib);

    // Fixed-point division: (a << shift_amount) / safe_divisor
    // Since safe_divisor is multiple of 2^shift_amount, we can compute a / (safe_divisor >> shift_amount)
    let shift_const = builder.ins().iconst(target_type, shift_amount);
    let divisor_shifted = builder.ins().ushr(safe_divisor, shift_const);
    let div_result = builder.ins().sdiv(arg1, divisor_shifted);

    // Select: if divisor was zero, use saturation value, otherwise use division result
    // saturation_value = numerator_is_zero ? 0 : (is_negative ? min : max)
    // Use arithmetic: saturation_value = (numerator_is_zero * 0) + (!numerator_is_zero * infinity_value)
    // where infinity_value = is_negative ? min : max

    // First compute infinity_value = is_negative ? min : max
    // infinity_value = min + (max - min) * is_negative
    let min_max_diff = builder.ins().isub(max_fixed_16x16, min_fixed_16x16);
    let is_negative_i32 = builder.ins().uextend(types::I32, is_negative);
    let scaled_diff = builder.ins().imul(is_negative_i32, min_max_diff);
    let infinity_value = builder.ins().iadd(min_fixed_16x16, scaled_diff);

    // Now saturation_value = (numerator_is_zero * 0) + (!numerator_is_zero * infinity_value)
    // !numerator_is_zero = 1 - numerator_is_zero (since bool is 0/1)
    let one_const = builder.ins().iconst(target_type, 1);
    let numerator_is_zero_i32 = builder.ins().uextend(types::I32, numerator_is_zero);
    let not_numerator_zero_i32 = builder.ins().isub(one_const, numerator_is_zero_i32);
    let zero_contrib = builder.ins().imul(numerator_is_zero_i32, zero);
    let infinity_contrib = builder.ins().imul(not_numerator_zero_i32, infinity_value);
    let saturation_value = builder.ins().iadd(zero_contrib, infinity_contrib);

    // Final result: is_zero ? saturation_value : div_result
    let is_zero_i32 = builder.ins().uextend(types::I32, is_zero);
    let not_is_zero_i32 = builder.ins().isub(one_const, is_zero_i32);
    let zero_contrib_final = builder.ins().imul(is_zero_i32, saturation_value);
    let div_contrib = builder.ins().imul(not_is_zero_i32, div_result);
    let new_result = builder.ins().iadd(zero_contrib_final, div_contrib);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

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
