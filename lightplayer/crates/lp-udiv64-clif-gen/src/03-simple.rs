#![no_std]

/// Simplified version that isolates the i64 subtraction bug
/// This reproduces the exact pattern from block5 that causes the infinite loop
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    // Build 64-bit dividend
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;

    // Simplified: just do one subtraction and check if result >= 0
    // This isolates the bug in i64 subtraction and comparison
    let mut duo = duo;
    let div_shifted = div << 1; // Simple shift by 1
    
    // This is the problematic pattern:
    // 1. i64 subtraction
    // 2. Compare result >= 0
    // 3. Branch based on comparison
    loop {
        let sub = duo.wrapping_sub(div_shifted);
        if 0 <= (sub as i64) {
            duo = sub;
            // Exit after one iteration to avoid infinite loop
            break;
        }
        // Should never reach here if subtraction works correctly
        break;
    }
    
    // Return low 32 bits as "quotient"
    (duo as u32) / divisor
}

