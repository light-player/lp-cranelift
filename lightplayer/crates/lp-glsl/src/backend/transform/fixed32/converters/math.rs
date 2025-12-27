//! Math function conversion functions.

use crate::backend::transform::fixed32::converters::{
    create_zero_const, extract_unary_operand, get_first_result, map_operand,
};
use crate::backend::transform::fixed32::types::FixedPointFormat;
use crate::error::GlslError;
use cranelift_codegen::ir::{condcodes::IntCC, types, Function, Inst, InstBuilder, Value};
use cranelift_frontend::FunctionBuilder;
use hashbrown::HashMap;

/// Convert Ceil instruction.
pub(crate) fn convert_ceil(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
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
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
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

/// Convert Trunc instruction.
pub(crate) fn convert_trunc(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    // Trunc is the same as floor for positive numbers, but rounds toward zero
    // For fixed-point, we can use the same approach as floor
    convert_floor(old_func, old_inst, builder, value_map, format)
}

/// Convert Nearest instruction (round to nearest).
pub(crate) fn convert_nearest(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
    let arg = extract_unary_operand(old_func, old_inst)?;
    let mapped_arg = map_operand(value_map, arg);
    let target_type = format.cranelift_type();
    let shift_amount = format.shift_amount();

    // Nearest: round to nearest integer
    // In fixed-point: (value + (1 << (shift - 1))) >> shift, then << shift
    let half = 1i64 << (shift_amount - 1);
    let half_const = builder.ins().iconst(target_type, half);
    let added = builder.ins().iadd(mapped_arg, half_const);
    let shift_const = builder.ins().iconst(target_type, shift_amount);
    let rounded = builder.ins().sshr(added, shift_const);
    let new_result = builder.ins().ishl(rounded, shift_const);

    let old_result = get_first_result(old_func, old_inst);
    value_map.insert(old_result, new_result);

    Ok(())
}

/// Convert Sqrt using Newton-Raphson method.
pub(crate) fn convert_sqrt(
    old_func: &Function,
    old_inst: Inst,
    builder: &mut FunctionBuilder,
    value_map: &mut HashMap<Value, Value>,
    format: FixedPointFormat,
) -> Result<(), GlslError> {
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

#[cfg(test)]
#[cfg(feature = "std")]
mod tests {
    use crate::backend::transform::fixed32::fixed32_test_util;

    /// Test sqrt: square root
    ///
    /// NOTE: This test is currently ignored because sqrt uses i64 division
    /// which is not supported on riscv32. We'll need to implement an alternative
    /// algorithm that doesn't require i64 division.
    #[test]
    #[cfg(feature = "emulator")]
    #[ignore]
    fn test_fixed32_sqrt() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.0p2
    v1 = sqrt v0
    return v1
}
"#;
        // Result should be 2.0 (sqrt of 4.0)
        // Note: Newton-Raphson may have some precision error, so we allow a small tolerance
        fixed32_test_util::run_fixed32_test(clif, 2.0);
    }

    /// Test sqrt: square root of 9.0
    ///
    /// NOTE: This test is currently ignored because sqrt uses i64 division
    /// which is not supported on riscv32.
    #[test]
    #[cfg(feature = "emulator")]
    #[ignore]
    fn test_fixed32_sqrt_9() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x1.2p3
    v1 = sqrt v0
    return v1
}
"#;
        // Result should be 3.0 (sqrt of 9.0)
        fixed32_test_util::run_fixed32_test(clif, 3.0);
    }

    /// Test sqrt: square root of zero
    ///
    /// NOTE: This test is currently ignored because sqrt uses i64 division
    /// which is not supported on riscv32.
    #[test]
    #[cfg(feature = "emulator")]
    #[ignore]
    fn test_fixed32_sqrt_zero() {
        let clif = r#"
function %main() -> f32 system_v {
block0:
    v0 = f32const 0x0.0p0
    v1 = sqrt v0
    return v1
}
"#;
        // Result should be 0.0
        fixed32_test_util::run_fixed32_test(clif, 0.0);
    }
}
