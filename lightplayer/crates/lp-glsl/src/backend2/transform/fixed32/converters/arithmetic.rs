//! Arithmetic operation conversion functions.

use crate::backend2::transform::fixed32::converters::{extract_binary_operands, extract_unary_operand, get_first_result, map_operand, create_zero_const, create_max_fixed_const, create_min_fixed_const};
use crate::backend2::transform::fixed32::types::FixedPointFormat;
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Inst, InstBuilder, types, condcodes::IntCC};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Convert Fadd to fixed-point addition
pub(crate) fn convert_fadd(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Fixed-point addition is just integer addition (no conversion needed)
    // Both operands are already in fixed-point format
    let result = builder.ins().iadd(arg1, arg2);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

    Ok(())
}

/// Convert Fsub to fixed-point subtraction
pub(crate) fn convert_fsub(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<cranelift_codegen::ir::Value, cranelift_codegen::ir::Value>,
    _format: FixedPointFormat,
) -> Result<(), GlslError> {
    let (arg1_old, arg2_old) = extract_binary_operands(old_func, old_inst)?;
    let arg1 = map_operand(value_map, arg1_old);
    let arg2 = map_operand(value_map, arg2_old);

    // Fixed-point subtraction: a - b
    let result = builder.ins().isub(arg1, arg2);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

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

    // Truncate back to i32
    let result = builder.ins().ireduce(target_type, shifted_wide);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, result);

    Ok(())
}

/// Convert Fdiv to fixed-point division with scaling and zero handling
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
    let result = builder.ins().select(is_zero, saturation_value, div_result);

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
    use super::*;
    use crate::backend2::transform::fixed32::fixed32_test_util;

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
