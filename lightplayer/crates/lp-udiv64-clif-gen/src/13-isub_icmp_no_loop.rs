#![no_std]

/// Test 13: i64 subtraction followed by comparison, NO loop
/// Tests if the bug happens without loop (should pass if bug is loop-specific)
#[no_mangle]
pub fn divide64(dividend_lo: u32, divisor: u32) -> u32 {
    // Simple values with zero high parts
    let duo = dividend_lo as u64;
    let div_shifted = (divisor as u64) << 1;
    
    // Do subtraction
    let sub = duo.wrapping_sub(div_shifted);
    
    // Do comparison - no loop, just return the comparison result
    if 0 <= (sub as i64) {
        1
    } else {
        0
    }
}

// Correct expectations:
// run: divide64(100, 2) == 1   ; 0 <= (100 - 4) = 0 <= 96 (true)
// run: divide64(50, 100) == 0  ; 0 <= (50 - 200) = 0 <= -150 (false)
// run: divide64(0, 100) == 0   ; 0 <= (0 - 200) = 0 <= -200 (false)

