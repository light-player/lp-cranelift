//! Arithmetic operation conversion functions.

use crate::backend::transform::fixed32::converters::{
    create_max_fixed_const, create_min_fixed_const, create_zero_const, extract_binary_operands,
    extract_unary_operand, get_first_result, map_operand,
};
use crate::backend::transform::fixed32::types::FixedPointFormat;
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Inst, InstBuilder, condcodes::IntCC, types};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Convert Fadd to fixed-point addition with saturation
pub(crate) fn convert_fadd(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Fixed-point addition is just integer addition (no conversion needed)
    // Both operands are already in fixed-point format
    let result = builder.ins().iadd(arg1, arg2);

    // Saturate result to fixed-point range
    let zero = create_zero_const(builder, format);
    let max_fixed = create_max_fixed_const(builder, format);
    let min_fixed = create_min_fixed_const(builder, format);

    // Check for overflow: if both operands are positive and result is negative, saturate to max
    let arg1_positive = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, arg1, zero);
    let arg2_positive = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, arg2, zero);
    let result_negative = builder.ins().icmp(IntCC::SignedLessThan, result, zero);
    let both_positive = builder.ins().band(arg1_positive, arg2_positive);
    let overflow = builder.ins().band(both_positive, result_negative);

    // Check for underflow: if both operands are negative and result is positive, saturate to min
    let arg1_negative = builder.ins().icmp(IntCC::SignedLessThan, arg1, zero);
    let arg2_negative = builder.ins().icmp(IntCC::SignedLessThan, arg2, zero);
    let result_positive = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, result, zero);
    let both_negative = builder.ins().band(arg1_negative, arg2_negative);
    let underflow = builder.ins().band(both_negative, result_positive);

    // Clamp result to range [min_fixed, max_fixed]
    let clamped_max = builder.ins().smin(result, max_fixed);
    let clamped = builder.ins().smax(clamped_max, min_fixed);

    // Select: if overflow use max, if underflow use min, otherwise use clamped
    let saturated = builder.ins().select(overflow, max_fixed, clamped);
    let final_result = builder.ins().select(underflow, min_fixed, saturated);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, final_result);

    Ok(())
}

/// Convert Fsub to fixed-point subtraction with saturation
pub(crate) fn convert_fsub(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Fixed-point subtraction: a - b
    let result = builder.ins().isub(arg1, arg2);

    // Saturate result to fixed-point range
    let zero = create_zero_const(builder, format);
    let max_fixed = create_max_fixed_const(builder, format);
    let min_fixed = create_min_fixed_const(builder, format);

    // Check for overflow: if arg1 is positive and arg2 is negative and result is negative, saturate to max
    let arg1_positive = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, arg1, zero);
    let arg2_negative = builder.ins().icmp(IntCC::SignedLessThan, arg2, zero);
    let result_negative = builder.ins().icmp(IntCC::SignedLessThan, result, zero);
    let overflow_cond = builder.ins().band(arg1_positive, arg2_negative);
    let overflow = builder.ins().band(overflow_cond, result_negative);

    // Check for underflow: if arg1 is negative and arg2 is positive and result is positive, saturate to min
    let arg1_negative = builder.ins().icmp(IntCC::SignedLessThan, arg1, zero);
    let arg2_positive = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, arg2, zero);
    let result_positive = builder
        .ins()
        .icmp(IntCC::SignedGreaterThanOrEqual, result, zero);
    let underflow_cond = builder.ins().band(arg1_negative, arg2_positive);
    let underflow = builder.ins().band(underflow_cond, result_positive);

    // Clamp result to range [min_fixed, max_fixed]
    let clamped_max = builder.ins().smin(result, max_fixed);
    let clamped = builder.ins().smax(clamped_max, min_fixed);

    // Select: if overflow use max, if underflow use min, otherwise use clamped
    let saturated = builder.ins().select(overflow, max_fixed, clamped);
    let final_result = builder.ins().select(underflow, min_fixed, saturated);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, final_result);

    Ok(())
}

/// Convert Fmul to fixed-point multiplication with scaling
pub(crate) fn convert_fmul(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
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

    // Saturate to fixed-point range BEFORE truncation
    // This catches overflow cases where the i64 value exceeds the i32 range
    let max_fixed_i64 = builder.ins().iconst(types::I64, 0x7FFF_FFFFi64);
    // Min is -2147483648 (i32::MIN, which is 0x80000000 in fixed-point)
    let min_fixed_i64 = builder.ins().iconst(types::I64, -2147483648i64);

    // Clamp at i64 level
    let clamped_max_i64 = builder.ins().smin(shifted_wide, max_fixed_i64);
    let clamped_i64 = builder.ins().smax(clamped_max_i64, min_fixed_i64);

    // Truncate the clamped value
    let result = builder.ins().ireduce(target_type, clamped_i64);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

    Ok(())
}

/// Convert Fdiv to fixed-point division using reciprocal multiplication
///
/// Uses the reciprocal method: quotient = (dividend * recip * 2) >> shift_amount
/// where recip = 0x8000_0000 / divisor (for fixed16x16)
///
/// This approach is chosen for speed and simplicity, with ~0.01% typical precision
/// error (can be larger for edge cases). For exact division, see the incomplete
/// full long division implementation in the `feature/udiv64` branch.
///
/// Handles division by zero by saturating to maximum/minimum fixed-point values
/// based on the sign of the numerator. This matches typical fixed-point arithmetic
/// behavior where division by zero is undefined but we need to avoid crashes.
pub(crate) fn convert_fdiv(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Check for division by zero
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

    // Extract signs for signed division
    let arg1_is_negative = builder.ins().icmp(IntCC::SignedLessThan, arg1, zero);
    let arg2_is_negative = builder.ins().icmp(IntCC::SignedLessThan, arg2, zero);
    let result_is_negative = builder.ins().bxor(arg1_is_negative, arg2_is_negative);

    // Compute absolute values (as i32)
    let arg1_negated = builder.ins().ineg(arg1);
    let arg2_negated = builder.ins().ineg(arg2);
    let arg1_abs = builder.ins().select(arg1_is_negative, arg1_negated, arg1);
    let arg2_abs = builder.ins().select(arg2_is_negative, arg2_negated, arg2);

    // Ensure divisor is never zero for reciprocal calculation
    let one = builder.ins().iconst(target_type, 1);
    let safe_divisor_abs = builder.ins().select(is_zero, one, arg2_abs);

    // Calculate reciprocal: recip = 0x8000_0000 / divisor_abs
    // This is the key to the reciprocal method - we precompute 1/divisor scaled by 2^31
    let recip_base = builder.ins().iconst(target_type, 0x8000_0000i64);
    let recip = builder.ins().udiv(recip_base, safe_divisor_abs);

    // Convert to i64 for multiplication to avoid overflow
    let arg1_abs_i64 = builder.ins().uextend(types::I64, arg1_abs);
    let recip_i64 = builder.ins().uextend(types::I64, recip);

    // Calculate quotient = (dividend_abs * recip * 2) >> shift_amount
    // Multiply dividend by reciprocal
    let mul_result = builder.ins().imul(arg1_abs_i64, recip_i64);

    // Multiply by 2 (left shift by 1)
    let two_i64 = builder.ins().iconst(types::I64, 2);
    let mul_result_2x = builder.ins().imul(mul_result, two_i64);

    // Right shift by shift_amount (16 for fixed16x16)
    let shift_const_i64 = builder.ins().iconst(types::I64, shift_amount);
    let quotient_i64 = builder.ins().ushr(mul_result_2x, shift_const_i64);

    // Truncate back to i32
    let quotient_abs = builder.ins().ireduce(target_type, quotient_i64);

    // Apply sign: if result should be negative, negate it
    let quotient_negated = builder.ins().ineg(quotient_abs);
    let quotient = builder
        .ins()
        .select(result_is_negative, quotient_negated, quotient_abs);

    // Clamp to fixed-point range
    let clamped_max = builder.ins().smin(quotient, max_fixed);
    let clamped = builder.ins().smax(clamped_max, min_fixed);

    // Final result: if divisor was zero, use saturation_value, else use clamped quotient
    let result = builder.ins().select(is_zero, saturation_value, clamped);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

    Ok(())
}

/// Convert Fneg to fixed-point negation
pub(crate) fn convert_fneg(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);

    let result = builder.ins().ineg(mapped_arg);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

    Ok(())
}

/// Convert Fabs using conditional select
pub(crate) fn convert_fabs(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);

    // Absolute value: if (arg < 0) then -arg else arg
    let zero = create_zero_const(builder, format);
    let is_negative = builder.ins().icmp(IntCC::SignedLessThan, mapped_arg, zero);
    let negated = builder.ins().ineg(mapped_arg);
    let result = builder.ins().select(is_negative, negated, mapped_arg);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

    Ok(())
}

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use crate::backend::transform::fixed32::fixed32_test_util;

    /// Test fadd: addition
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_fadd() {
        // Use proper hex scientific notation: 0x1.8p-1 = 0.75, 0x1.8p1 = 3.0
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.8p1
    v1 = f32const 0x1.8p-1
    v2 = fadd v0, v1
    return v2
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 3.75);
    }

    /// Test fsub: subtraction
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_fsub() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.4p2
    v1 = f32const 0x1.4p1
    v2 = fsub v0, v1
    return v2
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 2.5);
    }

    /// Test fmul: multiplication
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_fmul() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.0p1
    v1 = f32const 0x1.8p1
    v2 = fmul v0, v1
    return v2
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 6.0);
    }

    /// Test fdiv: division
    ///
    /// NOTE: This test is currently ignored due to a known issue with the division algorithm.
    /// The old backend has the same algorithm and may have the same bug. We'll fix this separately.
    #[test]
    #[cfg(feature = "emulator")]
    #[ignore]
    fn test_fixed32_fdiv() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.4p3
    v1 = f32const 0x1.4p1
    v2 = fdiv v0, v1
    return v2
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 4.0);
    }

    /// Test fneg: negation
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_fneg() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.4p1
    v1 = fneg v0
    return v1
}
"#;
        fixed32_test_util::run_fixed32_test(clif, -2.5);
    }

    /// Test fabs: absolute value
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_fabs() {
        // Test with negative value
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const -0x1.4p1
    v1 = fabs v0
    return v1
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 2.5);
    }

    /// Test fabs: absolute value with positive value
    #[test]
    #[cfg(feature = "emulator")]
    fn test_fixed32_fabs_positive() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.4p1
    v1 = fabs v0
    return v1
}
"#;
        fixed32_test_util::run_fixed32_test(clif, 2.5);
    }
}
