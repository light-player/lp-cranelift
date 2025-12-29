#![no_std]

/// Minimal division test that isolates the i64 subtraction bug
/// This reproduces the exact failing pattern with minimal code
#[no_mangle]
pub fn divide64(dividend_lo: u32, divisor: u32) -> u32 {
    // Build 64-bit dividend (high part is 0)
    let duo = dividend_lo as u64;
    let div = divisor as u64;
    
    // Simple shift - just shift by 1 to keep it simple
    let div_shifted = div << 1;
    
    // The problematic loop pattern from block5:
    // - i64 subtraction: sub = duo - div_shifted
    // - Signed comparison: if 0 <= sub
    // - Update duo if positive
    
    let mut duo = duo;
    let mut iterations = 0;
    const MAX_ITER: u32 = 10; // Prevent infinite loop
    
    while iterations < MAX_ITER {
        let sub = duo.wrapping_sub(div_shifted);
        
        // This is the exact pattern that fails:
        // The comparison 0 <= (sub as i64) is wrong when sub's high part is corrupted
        if 0 <= (sub as i64) {
            duo = sub;
            iterations += 1;
        } else {
            // Result is negative, we're done
            break;
        }
        
        // Right-shift divisor for next iteration (simplified)
        let div_shifted = div_shifted >> 1;
        if div_shifted == 0 {
            break;
        }
    }
    
    // Return a simple result based on iterations
    // This helps verify the loop behavior
    if iterations == MAX_ITER {
        // Hit max iterations - indicates infinite loop bug
        0xdeadbeef
    } else {
        iterations
    }
}

