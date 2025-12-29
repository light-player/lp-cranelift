//! 64-bit dividend by 64-bit divisor division with remainder.
//!
//! This implementation is adapted from Rust's compiler-builtins crate, specifically
//! the `impl_delegate!` macro for u64 division in `compiler-builtins/src/int/specialized_div_rem/delegate.rs`.
//!
//! Original source: https://github.com/rust-lang/compiler-builtins
//!
//! The algorithm uses a combination of hardware division (when available) and binary
//! long division to divide a 64-bit unsigned dividend by a 64-bit unsigned divisor,
//! returning both the quotient and remainder as a tuple `(u64, u64)`.
//!
//! License: Rust's compiler-builtins is dual-licensed under Apache 2.0 and MIT,
//! which is compatible with this project's Apache 2.0 license.

use crate::builtins::u64_by_u32_div_rem::__lp_u64_by_u32_div_rem;

/// Normalization shift function for 32-bit values.
///
/// Finds the shift left that the divisor `div` would need to be normalized for a binary
/// long division step with the dividend `duo`.
#[inline(always)]
fn u32_normalization_shift(duo: u32, div: u32, full_normalization: bool) -> usize {
    let mut shl = (div.leading_zeros() - duo.leading_zeros()) as usize;
    if full_normalization {
        if duo < (div << shl) {
            shl -= 1;
        }
    }
    shl
}

/// Normalization shift function for 64-bit values (using u32 operations).
#[inline(always)]
fn u64_normalization_shift(duo_hi: u32, div_hi: u32, full_normalization: bool) -> usize {
    u32_normalization_shift(duo_hi, div_hi, full_normalization)
}

/// 32-bit by 32-bit division helper.
#[inline(always)]
fn u32_by_u32_div_rem(duo: u32, div: u32) -> (u32, u32) {
    (duo / div, duo % div)
}

/// Divide a 64-bit unsigned dividend by a 64-bit unsigned divisor.
/// Returns (quotient, remainder) as a tuple.
#[no_mangle]
pub extern "C" fn __lp_u64_by_u64_div_rem(dividend: u64, divisor: u64) -> (u64, u64) {
    if divisor == 0 {
        unsafe { core::hint::unreachable_unchecked() }
    }

    let duo_lo = dividend as u32;
    let duo_hi = (dividend >> 32) as u32;
    let div_lo = divisor as u32;
    let div_hi = (divisor >> 32) as u32;

    match (div_lo == 0, div_hi == 0, duo_hi == 0) {
        (true, true, _) => {
            // Division by zero
            unsafe { core::hint::unreachable_unchecked() }
        }
        (_, false, true) => {
            // `duo` < `div` - quotient is 0, remainder is dividend
            return (0, dividend);
        }
        (false, true, true) => {
            // Both dividend and divisor fit in 32 bits - delegate to 32-bit division
            let tmp = u32_by_u32_div_rem(duo_lo, div_lo);
            return (tmp.0 as u64, tmp.1 as u64);
        }
        (false, true, false) => {
            // 64-bit dividend, 32-bit divisor
            if duo_hi < div_lo {
                // `quo_hi` will always be 0. This performs a binary long division algorithm
                // to zero `duo_hi` followed by a half division.
                let norm_shift = u32_normalization_shift(div_lo, duo_hi, false);
                let n = 32;
                let shl = if norm_shift == 0 {
                    n - 1
                } else {
                    n - norm_shift
                };

                let mut div_shifted: u64 = divisor << shl;
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
                            // Delegate to get the rest of the quotient
                            let (quo_rest, rem) = __lp_u64_by_u32_div_rem(duo, div_lo);
                            return ((quo_lo | quo_rest) as u64, rem as u64);
                        }
                    }
                    div_shifted >>= 1;
                    pow_lo >>= 1;
                }
            } else if duo_hi == div_lo {
                // `quo_hi == 1`
                let (quo_rest, rem) = __lp_u64_by_u32_div_rem(dividend, divisor as u32);
                return ((1u64 << 32) | (quo_rest as u64), rem as u64);
            } else {
                // `div_lo < duo_hi`
                let n_h = 16;
                if (div_lo >> n_h) == 0 {
                    // Short division of u64 by a u16
                    let div_0 = div_lo as u16 as u32;
                    let (quo_hi, rem_3) = u32_by_u32_div_rem(duo_hi, div_0);

                    let duo_mid = ((dividend >> n_h) as u16 as u32) | (rem_3 << n_h);
                    let (quo_1, rem_2) = u32_by_u32_div_rem(duo_mid, div_0);

                    let duo_lo = (dividend as u16 as u32) | (rem_2 << n_h);
                    let (quo_0, rem_1) = u32_by_u32_div_rem(duo_lo, div_0);

                    return (
                        (quo_0 as u64) | ((quo_1 as u64) << n_h) | ((quo_hi as u64) << 32),
                        rem_1 as u64,
                    );
                }

                // Short division composed of a half division for the hi part,
                // specialized 3 variable binary long division in the middle,
                // and another half division for the lo part.
                let duo_lo = dividend as u32;
                let tmp = u32_by_u32_div_rem(duo_hi, div_lo);
                let quo_hi = tmp.0;
                let mut duo = (duo_lo as u64) | ((tmp.1 as u64) << 32);
                if duo < divisor {
                    return (((quo_hi as u64) << 32), duo);
                }

                // The half division handled all shift alignments down to n, so this
                // division can continue with a shift of n - 1.
                let n = 32;
                let mut div_shifted: u64 = divisor << (n - 1);
                let mut pow_lo: u32 = 1 << (n - 1);
                let mut quo_lo: u32 = 0;
                loop {
                    let sub = duo.wrapping_sub(div_shifted);
                    if 0 <= (sub as i64) {
                        duo = sub;
                        quo_lo |= pow_lo;
                        let duo_hi = (duo >> 32) as u32;
                        if duo_hi == 0 {
                            // Delegate to get the rest of the quotient
                            let (quo_rest, rem) = __lp_u64_by_u32_div_rem(duo, div_lo);
                            return (
                                (quo_rest as u64) | (quo_lo as u64) | ((quo_hi as u64) << 32),
                                rem as u64,
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
            if dividend < divisor {
                return (0, dividend);
            }
            let div_original = divisor;
            let shl = u64_normalization_shift(duo_hi, div_hi, false);
            let mut duo = dividend;
            let mut div_shifted: u64 = divisor << shl;
            let mut pow_lo: u32 = 1u32.wrapping_shl(shl as u32);
            let mut quo_lo: u32 = 0;
            loop {
                let sub = duo.wrapping_sub(div_shifted);
                if 0 <= (sub as i64) {
                    duo = sub;
                    quo_lo |= pow_lo;
                    if duo < div_original {
                        return (quo_lo as u64, duo);
                    }
                }
                div_shifted >>= 1;
                pow_lo >>= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    extern crate std;
    use super::__lp_u64_by_u64_div_rem;
    use std::vec;
    use std::vec::Vec;

    #[test]
    fn test_range() {
        let cases: Vec<(u64, u64)> = vec![
            (0, 1),
            (1, 1),
            (100, 5),
            (100, 3),
            (1000, 7),
            (0xFFFF_FFFF, 1),
            (0xFFFF_FFFF, 2),
            (0xFFFF_FFFF, 3),
            (0xFFFF_FFFF, 0xFFFF_FFFF),
            (0x1_0000_0000, 1),
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
            // Test 64-bit divisors
            (0x1_0000_0000, 0x1_0000_0000),
            (0xFFFF_FFFF_FFFF_FFFF, 0x1_0000_0000),
            (0x8000_0000_0000_0000, 0x8000_0000_0000_0000),
        ];

        let mut has_failures = false;

        for (dividend, divisor) in cases {
            let (actual_quo, actual_rem) = __lp_u64_by_u64_div_rem(dividend, divisor);

            let expected_quo = dividend / divisor;
            let expected_rem = dividend % divisor;

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

