#![no_std]

/// Test 09: i64 subtraction when both operands have zero high parts
/// This is the common case in the division loop
#[no_mangle]
pub fn divide64(dividend_lo: u32, divisor: u32) -> u64 {
    // Both values have zero high parts (common in the loop)
    let duo = dividend_lo as u64;  // High part is 0
    let div_shifted = (divisor as u64) << 1;  // High part is 0
    
    // Do subtraction - this should work correctly
    let sub = duo.wrapping_sub(div_shifted);
    
    // Return the high 32 bits to check if they're corrupted
    (sub >> 32) as u32 as u64
}

// run: divide64(0, 100) == 0
// run: divide64(100, 2) == 0
// run: divide64(50, 100) == 0

