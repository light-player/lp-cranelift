//! Arithmetic operation conversion functions.

use crate::backend::transform::fixed32::types::FixedPointFormat;
use crate::error::GlslError;

use cranelift_codegen::ir::{Function, Inst, InstBuilder, Value, condcodes::IntCC, types};
use cranelift_frontend::FunctionBuilder;

use crate::backend::ir_utils::fixed_point::{
    create_max_fixed_const, create_min_fixed_const, create_zero_const,
};
use crate::backend::ir_utils::instruction::{
    extract_binary_operands, extract_unary_operand, get_first_result,
};
use crate::backend::ir_utils::value_map::map_operand;

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

    // Check conditions for saturation value
    let numerator_is_zero = builder.ins().icmp(IntCC::Equal, arg1, zero);
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, arg1, zero);

    // Compute saturation value using select instructions
    let max_fixed = create_max_fixed_const(builder, format);
    let min_fixed = create_min_fixed_const(builder, format);

    // infinity_value = is_negative ? min_fixed : max_fixed
    let infinity_value = builder.ins().select(is_negative, min_fixed, max_fixed);

    // saturation_value = numerator_is_zero ? zero : infinity_value
    let saturation_value = builder
        .ins()
        .select(numerator_is_zero, zero, infinity_value);

    // Perform division if divisor is non-zero
    let shift_const = builder.ins().iconst(target_type, shift_amount);
    // Use signed shift right to preserve sign bit for negative divisors
    let divisor_shifted = builder.ins().sshr(arg2, shift_const);

    // Check if divisor_shifted became zero (bug fix for small divisors < 2^16)
    let divisor_shifted_is_zero = builder.ins().icmp(IntCC::Equal, divisor_shifted, zero);

    // Use a safe divisor for the shifted case to avoid division by zero
    let one = builder.ins().iconst(target_type, 1);
    let safe_divisor_shifted = builder
        .ins()
        .select(divisor_shifted_is_zero, one, divisor_shifted);

    // For normal case: arg1 / safe_divisor_shifted
    let div_by_shifted_divisor = builder.ins().sdiv(arg1, safe_divisor_shifted);

    // For small divisor case: (arg1 << shift_amount) / safe_arg2
    // Ensure arg2 is never zero for division to avoid SIGILL
    let safe_arg2 = builder.ins().select(is_zero, one, arg2);
    let arg1_shifted = builder.ins().ishl(arg1, shift_const);
    let div_by_full_divisor = builder.ins().sdiv(arg1_shifted, safe_arg2);

    // Select the result: if divisor_shifted_is_zero then div_by_full_divisor else div_by_shifted_divisor
    let div_result = builder.ins().select(
        divisor_shifted_is_zero,
        div_by_full_divisor,
        div_by_shifted_divisor,
    );

    // Final result: if divisor was zero, use saturation_value, else use div_result
    let new_result = builder.ins().select(is_zero, saturation_value, div_result);

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
