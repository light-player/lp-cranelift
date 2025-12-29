#![no_std]

/// Test 12: Minimal loop with i64 subtraction and comparison
/// Simplest possible test that reproduces the bug
#[no_mangle]
pub fn divide64(dividend_lo: u32, divisor: u32) -> u32 {
    // Start with simple values - both have zero high parts
    let mut duo = dividend_lo as u64;  // High part is 0
    let mut div_shifted = (divisor as u64) << 1;  // High part is 0
    
    // Minimal loop: just one iteration to test the pattern
    let mut count = 0;
    loop {
        if count >= 1 {
            break;
        }
        count += 1;
        
        // The problematic pattern:
        // 1. i64 subtraction
        let sub = duo.wrapping_sub(div_shifted);
        
        // 2. Signed comparison (this is where it fails)
        if 0 <= (sub as i64) {
            duo = sub;
        }
        
        div_shifted >>= 1;
    }
    
    // Return the high 32 bits to check if they're corrupted
    // Should be 0, but bug makes it non-zero
    (duo >> 32) as u32
}

// run: divide64(100, 2) == 0
// run: divide64(50, 100) == 0
// run: divide64(0, 100) == 0

