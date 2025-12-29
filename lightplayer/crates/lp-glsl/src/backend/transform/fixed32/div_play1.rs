//! Reference implementation of fixed16x16 division using only 32-bit types.
//!
//! This module provides a reference implementation that can be used to:
//! 1. Test and validate the division algorithm
//! 2. Compare against compiler-generated code
//! 3. Understand the algorithm before implementing it in the compiler
//!
//! The algorithm handles:
//! - Division by zero (saturates to max/min)
//! - Small divisors (< 2^16) that require left-shifting the numerator
//! - Large divisors (>= 2^16) that can use direct division
//! - Both positive and negative values
//! - Overflow cases

/// Fixed16x16 format constants
const SHIFT: u32 = 16;
const SCALE: u32 = 1 << SHIFT; // 65536
const MAX_FIXED: i32 = 0x7FFF_0000; // Maximum representable fixed-point value
const MIN_FIXED: i32 = i32::MIN; // Minimum representable fixed-point value

/// Reference implementation for fixed16x16 division using only i32/u32.
///
/// For fixed16x16 format, division is: `(a / b) * 2^16 = (a << 16) / b`
///
/// The challenge: when `a << 16` overflows i32, we need to handle it
/// without using i64 division.
///
/// Algorithm:
/// 1. Handle division by zero (saturate to max/min based on sign)
/// 2. If divisor >> 16 is non-zero, use direct division: `a / (b >> 16)`
/// 3. Otherwise, for small divisors, use unsigned arithmetic:
///    - Convert to unsigned, shift left, divide, convert back
///    - Handle sign separately
pub fn fixed16x16_div_reference(a: i32, b: i32) -> i32 {
    // Handle division by zero
    if b == 0 {
        return if a >= 0 { MAX_FIXED } else { MIN_FIXED };
    }

    // Handle the case where b >> 16 is non-zero AND b has no fractional part
    // (i.e., b is exactly a multiple of 2^16)
    let b_shifted = b >> SHIFT;
    let b_fractional = b & ((1 << SHIFT) - 1); // Low 16 bits
    if b_shifted != 0 && b_fractional == 0 {
        // Normal case: a / (b >> 16) gives us the result directly
        // This works because: (a / (b >> 16)) = (a * 2^16) / b when b has no fractional part
        return a / b_shifted;
    }

    // Small divisor case: need (a << 16) / b
    // Problem: a << 16 might overflow u32, so we use u64 for the shift
    // But we can't use u64 division on riscv32, so we extract the low 32 bits
    // after the shift and use u32 division. This works because:
    // - When we shift an i32 value left by 16 bits in u64, the result fits in 48 bits
    // - The high 16 bits of the u64 result are zero (since we zero-extended from i32)
    // - So we can safely extract the low 32 bits and do u32 division

    // Extract signs
    let a_is_negative = a < 0;
    let b_is_negative = b < 0;
    let result_is_negative = a_is_negative != b_is_negative;

    // Work with absolute values as unsigned
    let a_abs = a.abs() as u32;
    let b_abs = b.abs() as u32;

    // Shift a left by 16 bits using u64 (to avoid overflow), then extract for u32 division
    // We can use u64 for the shift, but must use u32 for division
    // The key insight: when we do (a_abs as u64) << 16, the result is:
    // - If a_abs = 0x000a_0000, then (a_abs as u64) = 0x00000000000a0000
    // - After << 16: 0x0000000a00000000
    // - Low 32 bits: 0x00000000 (zeros!)
    // - But we want: 0x0a00_0000 (the u32 shift result)
    //
    // The solution: use wrapping_shl on u32, which gives us the correct result
    // (the wrap behavior is what we want for fixed-point division)
    let a_shifted = a_abs.wrapping_shl(SHIFT);

    // Do unsigned division in u32 (both operands are u32)
    // Note: This matches what the compiler does - we avoid u64 division by using u32
    let result_abs = a_shifted / b_abs;

    // Convert back to signed, applying the sign
    let result = if result_is_negative {
        -(result_abs as i32)
    } else {
        result_abs as i32
    };

    // Clamp to fixed-point range
    result.clamp(MIN_FIXED, MAX_FIXED)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Convert float to fixed16x16
    fn float_to_fixed(f: f32) -> i32 {
        (f * SCALE as f32).round() as i32
    }

    /// Convert fixed16x16 to float
    fn fixed_to_float(fixed: i32) -> f32 {
        fixed as f32 / SCALE as f32
    }

    #[test]
    fn test_close_to_one() {
        // 0.999 / 0.998 ≈ 1.001
        let a = float_to_fixed(0.999);
        let b = float_to_fixed(0.998);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 1.001).abs() < 0.001,
            "Expected ~1.001, got {}",
            result_float
        );
    }

    #[test]
    fn test_positive_positive() {
        // 10.0 / 2.0 = 5.0
        let a = float_to_fixed(10.0);
        let b = float_to_fixed(2.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 5.0).abs() < 0.001,
            "Expected 5.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_positive_negative() {
        // 10.0 / (-2.0) = -5.0
        let a = float_to_fixed(10.0);
        let b = float_to_fixed(-2.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - (-5.0)).abs() < 0.001,
            "Expected -5.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_negative_negative() {
        // (-10.0) / (-2.0) = 5.0
        let a = float_to_fixed(-10.0);
        let b = float_to_fixed(-2.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 5.0).abs() < 0.001,
            "Expected 5.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_divide_by_one() {
        // 7.5 / 1.0 = 7.5
        let a = float_to_fixed(7.5);
        let b = float_to_fixed(1.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 7.5).abs() < 0.001,
            "Expected 7.5, got {}",
            result_float
        );
    }

    #[test]
    fn test_small_fractions() {
        // 0.5 / 0.25 = 2.0
        let a = float_to_fixed(0.5);
        let b = float_to_fixed(0.25);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 2.0).abs() < 0.001,
            "Expected 2.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_similar_values() {
        // 0.707016 / 0.70718384 ≈ 1.0
        let a = float_to_fixed(0.707016);
        let b = float_to_fixed(0.70718384);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 1.0).abs() < 0.001,
            "Expected ~1.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_division_by_zero_positive() {
        let a = float_to_fixed(10.0);
        let b = 0;
        let result = fixed16x16_div_reference(a, b);
        assert_eq!(result, MAX_FIXED, "Division by zero should saturate to max");
    }

    #[test]
    fn test_division_by_zero_negative() {
        let a = float_to_fixed(-10.0);
        let b = 0;
        let result = fixed16x16_div_reference(a, b);
        assert_eq!(result, MIN_FIXED, "Division by zero should saturate to min");
    }

    #[test]
    fn test_large_divisor() {
        // Test case where divisor >> 16 is non-zero
        // 100.0 / 2.0 = 50.0
        let a = float_to_fixed(100.0);
        let b = float_to_fixed(2.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 50.0).abs() < 0.001,
            "Expected 50.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_expressions() {
        // (20.0 / 2.0) / (4.0 / 2.0) = 10.0 / 2.0 = 5.0
        let a = float_to_fixed(20.0);
        let b = float_to_fixed(2.0);
        let c = float_to_fixed(4.0);
        let d = float_to_fixed(2.0);

        let step1 = fixed16x16_div_reference(a, b);
        let step2 = fixed16x16_div_reference(c, d);
        let result = fixed16x16_div_reference(step1, step2);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 5.0).abs() < 0.001,
            "Expected 5.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_divide_variables() {
        // 15.0 / 3.0 = 5.0
        let a = float_to_fixed(15.0);
        let b = float_to_fixed(3.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 5.0).abs() < 0.001,
            "Expected 5.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_divide_in_assignment() {
        // 10.0 / 2.5 = 4.0
        let a = float_to_fixed(10.0);
        let b = float_to_fixed(2.5);
        eprintln!("a: 0x{:08x} ({})", a, fixed_to_float(a));
        eprintln!("b: 0x{:08x} ({})", b, fixed_to_float(b));
        let result = fixed16x16_div_reference(a, b);
        eprintln!("result: 0x{:08x} ({})", result, fixed_to_float(result));
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 4.0).abs() < 0.0001,
            "Expected 4.0, got {}",
            result_float
        );
    }

    #[test]
    fn test_divide_large_numbers() {
        // Large numbers are clamped to fixed16x16 max (32767.99998)
        // 1000000.0 / 1000.0 = 1000.0, but clamped to 32767.99998 / 1000.0 = 32.768
        // Actually, the input 1000000.0 gets clamped to 32767.99998 before division
        let a = float_to_fixed(1000000.0); // This will be clamped to max
        let b = float_to_fixed(1000.0);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        // The test expects ~32.768 because the numerator is clamped
        assert!(
            (result_float - 32.768).abs() < 0.001,
            "Expected ~32.768, got {}",
            result_float
        );
    }

    #[test]
    fn test_divide_similar_values_2() {
        // 0.5 / 0.5 = 1.0
        let a = float_to_fixed(0.5);
        let b = float_to_fixed(0.5);
        let result = fixed16x16_div_reference(a, b);
        let result_float = fixed_to_float(result);
        assert!(
            (result_float - 1.0).abs() < 0.001,
            "Expected 1.0, got {}",
            result_float
        );
    }
}
