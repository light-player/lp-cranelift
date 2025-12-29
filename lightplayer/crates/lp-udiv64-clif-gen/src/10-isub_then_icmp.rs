#![no_std]

/// Test 10: Subtraction followed immediately by comparison
/// Tests the exact pattern from block5 without loop
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    let norm_shift = div.leading_zeros() - (duo >> 32).leading_zeros();
    let shl = if norm_shift == 0 { 31 } else { 32 - norm_shift as usize };
    let div_shifted = div << shl;
    
    // Exact pattern from block5:
    // v33 = isub v31, v32
    // v35 = icmp sle v34, v33
    let sub = duo.wrapping_sub(div_shifted);
    let zero = 0i64;
    let is_positive = zero <= (sub as i64);
    
    if is_positive {
        1
    } else {
        0
    }
}

// run: divide64(0, 0, 100) == 0
// run: divide64(0, 100, 2) == 1
// run: divide64(0, 16, 2) == 0

