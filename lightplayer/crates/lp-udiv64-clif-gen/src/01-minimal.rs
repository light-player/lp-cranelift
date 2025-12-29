#![no_std]

/// Minimal test case: just i64 subtraction and signed comparison
/// This is the absolute minimum to reproduce the bug
#[no_mangle]
pub fn divide64(dividend_lo: u32, divisor: u32) -> u32 {
    // Create two i64 values with zero high parts
    let duo = dividend_lo as u64;
    let div_shifted = (divisor as u64) << 1;
    
    // The exact problematic pattern:
    // 1. i64 subtraction: sub = duo - div_shifted
    // 2. Signed comparison: 0 <= sub
    // This matches block5's v33 = isub v31, v32 and v35 = icmp sle v34, v33
    
    let sub = duo.wrapping_sub(div_shifted);
    let is_positive = 0 <= (sub as i64);
    
    // Return 1 if positive, 0 if negative
    if is_positive {
        1
    } else {
        0
    }
}

