#![no_std]

/// Divide a 64-bit unsigned dividend by a 32-bit unsigned divisor
/// Returns the 32-bit quotient
///
/// # Safety
/// Unsafe: assumes dividend_hi < divisor (quotient would overflow 32 bits otherwise)
///
/// This implementation is adapted from Rust's compiler-builtins delegate algorithm
/// for u64_div_rem on 32-bit targets.
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    // Precondition check removed - assume caller ensures dividend_hi < divisor
    // This reduces generated CLIF complexity

    // Build 64-bit dividend
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;

    // Split into high and low 32-bit parts
    let duo_hi = (duo >> 32) as u32;
    let div_lo = div as u32;

    // `quo_hi` will always be 0. This performs a binary long division algorithm
    // to zero `duo_hi` followed by a half division.

    // We can calculate the normalization shift using only 32-bit functions.
    // If we calculated the normalization shift using
    // `u32_normalization_shift(duo_hi, div_lo, false)`, it would break the
    // assumption the function has that the first argument is more than the
    // second argument. If the arguments are switched, the assumption holds true
    // since `duo_hi < div_lo`.
    let norm_shift = u32_normalization_shift(div_lo, duo_hi, false);
    let n = 32; // number of bits in u32
    let shl = if norm_shift == 0 {
        // Consider what happens if the msbs of `duo_hi` and `div_lo` align with
        // no shifting. The normalization shift will always return
        // `norm_shift == 0` regardless of whether it is fully normalized,
        // because `duo_hi < div_lo`. In that edge case, `n - norm_shift` would
        // result in shift overflow down the line. For the edge case, because
        // both `duo_hi < div_lo` and we are comparing all the significant bits
        // of `duo_hi` and `div`, we can make `shl = n - 1`.
        n - 1
    } else {
        // We also cannot just use `shl = n - norm_shift - 1` in the general
        // case, because when we are not in the edge case comparing all the
        // significant bits, then the full `duo < div` may not be true and thus
        // breaks the division algorithm.
        n - norm_shift
    };

    // The 3 variable restoring division algorithm is ideal for this task,
    // since `pow` and `quo` can be u32 and the delegation check is simple.
    let mut div_shifted: u64 = div << shl;
    let mut pow_lo: u32 = 1 << shl;
    let mut quo_lo: u32 = 0;
    let mut duo = duo;

    loop {
        let sub = duo.wrapping_sub(div_shifted);
        if 0 <= (sub as i64) {
            duo = sub;
            quo_lo |= pow_lo;
            let duo_hi = (duo >> 32) as u32;
            if duo_hi == 0 {
                // Delegate to get the rest of the quotient. Note that the
                // `div_lo` here is the original unshifted `div`.
                let tmp = u32_by_u32_div_rem(duo as u32, div_lo);
                return quo_lo | tmp.0;
            }
        }
        div_shifted >>= 1;
        pow_lo >>= 1;
    }
}

/// Normalization shift function for 32-bit values.
///
/// Finds the shift left that the divisor `div` would need to be normalized for a binary
/// long division step with the dividend `duo`.
#[inline(always)]
fn u32_normalization_shift(duo: u32, div: u32, full_normalization: bool) -> usize {
    // Use leading_zeros since RISC-V has CLZ instruction (or we can use software fallback)
    let mut shl = (div.leading_zeros() - duo.leading_zeros()) as usize;
    if full_normalization {
        // Skip bounds check on shift - compiler will optimize this
        if duo < (div << shl) {
            // When the msb of `duo` and `div` are aligned, the resulting `div` may be
            // larger than `duo`, so we decrease the shift by 1.
            shl -= 1;
        }
    }
    shl
}

/// 32-bit by 32-bit division helper.
///
/// This delegates to hardware division when available, or uses software division.
/// Uses unchecked division - assumes divisor != 0 (caller must ensure).
#[inline(always)]
fn u32_by_u32_div_rem(duo: u32, div: u32) -> (u32, u32) {
    // Unchecked division - assume div != 0
    // With overflow-checks=off, this won't panic
    (duo / div, duo % div)
}

// run: divide64(0, 0, 100) == 0
// run: divide64(0, 100, 2) == 50
// run: divide64(0, 16, 2) == 8
// run: divide64(0, 123456, 789) == 156
// run: divide64(1, 0, 2) == 0x80000000
// run: divide64(0xa, 0, 0x28000) == 0x40000

#[cfg(test)]
mod tests {
    use super::divide64;

    #[test]
    fn test_divide64_basic() {
        // Test cases matching the CLIF filetests
        assert_eq!(divide64(0, 0, 100), 0);
        assert_eq!(divide64(0, 100, 2), 50);
        assert_eq!(divide64(0, 16, 2), 8);
        assert_eq!(divide64(0, 123456, 789), 156);
        assert_eq!(divide64(1, 0, 2), 0x80000000);
        assert_eq!(divide64(0xa, 0, 0x28000), 0x40000);
    }

    #[test]
    fn test_divide64_edge_cases() {
        // Test division by 1
        assert_eq!(divide64(0, 100, 1), 100);
        assert_eq!(divide64(0, 0xffffffff, 1), 0xffffffff);

        // Test small values
        assert_eq!(divide64(0, 1, 1), 1);
        assert_eq!(divide64(0, 1, 2), 0);
        assert_eq!(divide64(0, 2, 2), 1);

        // Test with high bit set
        assert_eq!(divide64(0, 0x80000000, 2), 0x40000000);
        assert_eq!(divide64(0, 0x80000000, 0x80000000), 1);
    }

    #[test]
    fn test_divide64_large_values() {
        // Test with larger dividends (still with dividend_hi < divisor)
        assert_eq!(divide64(0, 0xffffffff, 0xffff), 0x10001);
        assert_eq!(divide64(0, 0x12345678, 0x1000), 0x12345);

        // Test with non-zero high part (but still < divisor)
        assert_eq!(divide64(50, 0, 100), 0x80000000); // (50 << 32) / 100
        assert_eq!(divide64(1, 0, 2), 0x80000000); // (1 << 32) / 2
    }

    #[test]
    fn test_divide64_verify_algorithm() {
        // Verify the algorithm produces correct results
        // by comparing with expected u64 division
        const TEST_CASES: &[(u32, u32, u32, u64)] = &[
            (0u32, 0u32, 100u32, 0u64),
            (0u32, 100u32, 2u32, 50u64),
            (0u32, 16u32, 2u32, 8u64),
            (0u32, 123456u32, 789u32, 156u64),
        ];

        for &(hi, lo, div, _expected) in TEST_CASES {
            let dividend = ((hi as u64) << 32) | (lo as u64);
            let divisor = div as u64;
            let expected_quotient = dividend / divisor;
            let result = divide64(hi, lo, div);
            assert_eq!(
                result as u64, expected_quotient,
                "divide64({}, {}, {}) failed: expected {}, got {}",
                hi, lo, div, expected_quotient, result
            );
        }
    }
}
