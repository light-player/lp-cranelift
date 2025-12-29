#![no_std]

/// Even simpler: just the subtraction and comparison pattern
/// This directly tests the bug without any division logic
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    // Build 64-bit values
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    // The exact pattern from block5:
    // - Start with duo (i64)
    // - Subtract div_shifted (i64)
    // - Check if result >= 0
    // - Loop based on comparison
    
    let mut duo = duo;
    let div_shifted = div << 1;
    let mut iterations = 0;
    
    // Limit iterations to prevent infinite loop
    while iterations < 10 {
        let sub = duo.wrapping_sub(div_shifted);
        if 0 <= (sub as i64) {
            duo = sub;
            iterations += 1;
            // Continue loop
        } else {
            // Result is negative, exit
            break;
        }
    }
    
    iterations
}

