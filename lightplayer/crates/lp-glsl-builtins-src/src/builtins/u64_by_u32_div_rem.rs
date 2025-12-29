//! 64-bit dividend by 32-bit divisor division with remainder.
//!
//! This implementation is adapted from Rust's compiler-builtins crate, specifically
//! the `u64_by_u32_div_rem` function in `compiler-builtins/src/int/specialized_div_rem/`.
//!
//! Original source: https://github.com/rust-lang/compiler-builtins
//!
//! The algorithm uses a combination of hardware division (when available) and binary
//! long division to divide a 64-bit unsigned dividend by a 32-bit unsigned divisor,
//! returning both the quotient and remainder as a tuple `(u32, u32)`.
//!
//! License: Rust's compiler-builtins is dual-licensed under Apache 2.0 and MIT,
//! which is compatible with this project's Apache 2.0 license.

#[no_mangle]
pub fn __lp_u64_by_u32_div_rem(dividend: u64, divisor: u32) -> (u32, u32) {
    // Precondition check removed - assume caller ensures dividend_hi < divisor
    // This reduces generated CLIF complexity

    let div = divisor as u64;

    // Split into high and low 32-bit parts
    let duo_lo = dividend as u32;
    let duo_hi = (dividend >> 32) as u32;
    let div_lo = div as u32;
    let div_hi = (div >> 32) as u32;

    // Match the cases from compiler-builtins delegate.rs
    // The algorithm handles different cases based on whether div_lo, div_hi, and duo_hi are zero
    match (div_lo == 0, div_hi == 0, duo_hi == 0) {
        (true, true, _) => {
            // Division by zero - should not happen, use unreachable
            unsafe { core::hint::unreachable_unchecked() }
        }
        (_, false, true) => {
            // `duo` < `div` - quotient is 0, remainder is dividend
            return (0, dividend as u32);
        }
        (false, true, true) => {
            // Both dividend and divisor fit in 32 bits - delegate to 32-bit division
            let tmp = u32_by_u32_div_rem(duo_lo, div_lo);
            return tmp;
        }
        (false, true, false) => {
            // 64-bit dividend, 32-bit divisor
            // This is our main case
            if duo_hi < div_lo {
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
                let mut duo = dividend;
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
                            return (quo_lo | tmp.0, tmp.1);
                        }
                    }
                    div_shifted >>= 1;
                    pow_lo >>= 1;
                }
            } else if duo_hi == div_lo {
                // `quo_hi == 1`. This branch is cheap and helps with edge cases.
                let tmp = u32_by_u32_div_rem(dividend as u32, div as u32);
                return (((1u64 << 32) | (tmp.0 as u64)) as u32, tmp.1);
            } else {
                // `div_lo < duo_hi`
                // `rem_hi == 0`
                let n_h = 16; // half the number of bits in u32
                if (div_lo >> n_h) == 0 {
                    // Short division of u64 by a u16, using u32 by u32 division
                    let div_0 = div_lo as u16 as u32;
                    let (quo_hi, rem_3) = u32_by_u32_div_rem(duo_hi, div_0);

                    let duo_mid = ((dividend >> n_h) as u16 as u32) | (rem_3 << n_h);
                    let (quo_1, rem_2) = u32_by_u32_div_rem(duo_mid, div_0);

                    let duo_lo = (dividend as u16 as u32) | (rem_2 << n_h);
                    let (quo_0, rem_1) = u32_by_u32_div_rem(duo_lo, div_0);

                    return (
                        ((quo_0 as u64) | ((quo_1 as u64) << n_h) | ((quo_hi as u64) << 32)) as u32,
                        rem_1,
                    );
                }

                // This is basically a short division composed of a half division for the hi
                // part, specialized 3 variable binary long division in the middle, and
                // another half division for the lo part.
                let duo_lo = dividend as u32;
                let tmp = u32_by_u32_div_rem(duo_hi, div_lo);
                let quo_hi = tmp.0;
                let mut duo = (duo_lo as u64) | ((tmp.1 as u64) << 32);
                // This check is required to avoid breaking the long division below.
                if duo < div {
                    return (((quo_hi as u64) << 32) as u32, duo as u32);
                }

                // The half division handled all shift alignments down to `n`, so this
                // division can continue with a shift of `n - 1`.
                let n = 32;
                let mut div_shifted: u64 = div << (n - 1);
                let mut pow_lo: u32 = 1 << (n - 1);
                let mut quo_lo: u32 = 0;
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
                            return (
                                (tmp.0 as u64 | quo_lo as u64 | ((quo_hi as u64) << 32)) as u32,
                                tmp.1,
                            );
                        }
                    }
                    div_shifted >>= 1;
                    pow_lo >>= 1;
                }
            }
        }
        (_, false, false) => {
            // Full 64-bit by 64-bit binary long division. `quo_hi` will always be 0.
            if dividend < div {
                return (0, dividend as u32);
            }
            let div_original = div;
            let shl = u32_normalization_shift(duo_hi, div_hi, false);
            let mut duo = dividend;
            let mut div_shifted: u64 = div << shl;
            let mut pow_lo: u32 = 1u32.wrapping_shl(shl as u32);
            let mut quo_lo: u32 = 0;
            loop {
                let sub = duo.wrapping_sub(div_shifted);
                if 0 <= (sub as i64) {
                    duo = sub;
                    quo_lo |= pow_lo;
                    if duo < div_original {
                        return (quo_lo, duo as u32);
                    }
                }
                div_shifted >>= 1;
                pow_lo >>= 1;
            }
        }
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

#[cfg(test)]
mod tests {
    #[cfg(test)]
    extern crate std;
    use super::__lp_u64_by_u32_div_rem;
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn test_range() {
        let cases: Vec<(u64, u32)> = vec![
            (0, 1),
            (1, 1),
            (100, 5),
            (100, 3),
            (1000, 7),
            (0xFFFF_FFFF, 1),
            (0xFFFF_FFFF, 2),
            (0xFFFF_FFFF, 3),
            (0xFFFF_FFFF, 0xFFFF_FFFF),
            (0x1_0000_0000, 2),
            (0x1_0000_0000, 4),
            (0x1_0000_0000, 3),
            (0xFFFF_FFFF_FFFF_FFFF, 1),
            (0xFFFF_FFFF_FFFF_FFFF, 2),
            (0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF),
            (0x8000_0000, 2),
            (0x8000_0000, 0x8000_0000),
            (100, 200), // dividend < divisor
            (100, 100), // dividend == divisor
        ];

        let mut has_failures = false;

        for (dividend, divisor) in cases {
            let (actual_quo, actual_rem) = __lp_u64_by_u32_div_rem(dividend, divisor);

            let expected_quo = (dividend / divisor as u64) as u32;
            let expected_rem = (dividend % divisor as u64) as u32;

            if actual_quo == expected_quo && actual_rem == expected_rem {
                std::println!("✓ {} / {} = {} rem {}", dividend, divisor, actual_quo, actual_rem);
            } else {
                std::println!(
                    "✗ {} / {} = {} rem {}; expected {} rem {}",
                    dividend,
                    divisor,
                    actual_quo,
                    actual_rem,
                    expected_quo,
                    expected_rem
                );
                has_failures = true;
            }
        }

        assert!(!has_failures);
    }
}
