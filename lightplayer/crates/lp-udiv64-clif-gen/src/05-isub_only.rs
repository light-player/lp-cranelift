#![no_std]

/// Test 05: Just i64 subtraction, no comparison, no loop
/// Isolates if the bug is in subtraction itself
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u64 {
    // Build 64-bit values exactly like the failing test
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    // Split into high and low 32-bit parts (like the failing test)
    let duo_hi = (duo >> 32) as u32;
    let div_lo = div as u32;
    
    // Calculate shift using the same logic as the failing test
    let norm_shift = u32_normalization_shift(div_lo, duo_hi, false);
    let n = 32;
    let shl = if norm_shift == 0 {
        n - 1
    } else {
        n - norm_shift
    };
    let div_shifted = div << shl;
    
    // Just do the subtraction, return the result
    let sub = duo.wrapping_sub(div_shifted);
    sub
}

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

// run: divide64(0, 0, 100) == 0xffffffff38000000
// run: divide64(0, 100, 2) == 0xffffffff80000064
// run: divide64(0, 16, 2) == 0xffffffff80000010

