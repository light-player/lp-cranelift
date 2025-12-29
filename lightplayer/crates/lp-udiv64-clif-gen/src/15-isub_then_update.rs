#![no_std]

/// Test 15: i64 subtraction, then update variable, then compare
/// Tests if updating the variable corrupts the high part
#[no_mangle]
pub fn divide64(dividend_lo: u32, divisor: u32) -> u32 {
    let mut duo = dividend_lo as u64;
    let div_shifted = (divisor as u64) << 1;
    
    // Do subtraction
    let sub = duo.wrapping_sub(div_shifted);
    
    // Update duo with the result
    duo = sub;
    
    // Now check if duo's high part is still correct
    // For positive results, high part should be 0
    // For negative results, high part should be 0xffffffff
    let duo_hi = (duo >> 32) as u32;
    
    // Also check the comparison
    let is_positive = 0 <= (duo as i64);
    
    // Return high part and comparison result
    // If positive: return high part (should be 0) | 0x10000
    // If negative: return high part (should be 0xffffffff)
    if is_positive {
        duo_hi | 0x10000  // Set bit 16 if positive
    } else {
        duo_hi  // Return high part (0xffffffff for negative)
    }
}

// run: divide64(100, 2) == 0x10000   ; 100 - 4 = 96 (positive), high=0, return 0|0x10000
// run: divide64(50, 100) == 0xffffffff  ; 50 - 200 = -150 (negative), high=0xffffffff
// run: divide64(0, 100) == 0xffffffff   ; 0 - 200 = -200 (negative), high=0xffffffff

