#![no_std]

/// Test 06: Just signed comparison, no subtraction
/// Isolates if the bug is in the comparison itself
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    // Build 64-bit values
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    // Calculate shift
    let norm_shift = div.leading_zeros() - (duo >> 32).leading_zeros();
    let shl = if norm_shift == 0 { 31 } else { 32 - norm_shift as usize };
    let div_shifted = div << shl;
    
    // Do subtraction
    let sub = duo.wrapping_sub(div_shifted);
    
    // Just do the comparison, return result
    if 0 <= (sub as i64) {
        1
    } else {
        0
    }
}

// run: divide64(0, 0, 100) == 0
// run: divide64(0, 100, 2) == 1
// run: divide64(0, 16, 2) == 0

