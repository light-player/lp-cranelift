#![no_std]

/// Test 07: Loop with exactly one iteration
/// Tests if the loop structure itself causes issues
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    let norm_shift = div.leading_zeros() - (duo >> 32).leading_zeros();
    let shl = if norm_shift == 0 { 31 } else { 32 - norm_shift as usize };
    let mut div_shifted = div << shl;
    let mut duo = duo;
    
    // Loop exactly once
    let mut iter_count = 0;
    loop {
        if iter_count >= 1 {
            break;
        }
        iter_count += 1;
        
        let sub = duo.wrapping_sub(div_shifted);
        if 0 <= (sub as i64) {
            duo = sub;
        }
        div_shifted >>= 1;
    }
    
    (duo as u32) / divisor
}

// run: divide64(0, 0, 100) == 0
// run: divide64(0, 100, 2) == 50
// run: divide64(0, 16, 2) == 8

