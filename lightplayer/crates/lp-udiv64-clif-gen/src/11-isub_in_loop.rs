#![no_std]

/// Test 11: i64 subtraction used in a loop (like block5)
/// This should reproduce the bug where the high part gets corrupted
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    let duo_hi = (duo >> 32) as u32;
    let div_lo = div as u32;
    let norm_shift = u32_normalization_shift(div_lo, duo_hi, false);
    let n = 32;
    let shl = if norm_shift == 0 { n - 1 } else { n - norm_shift };
    let mut div_shifted = div << shl;
    let mut duo = duo;
    
    // Loop with subtraction - this is where the bug manifests
    let mut iterations = 0;
    const MAX_ITER: u32 = 5;
    
    while iterations < MAX_ITER {
        let sub = duo.wrapping_sub(div_shifted);
        
        // Check if result >= 0 (this is where the bug shows up)
        if 0 <= (sub as i64) {
            duo = sub;  // Update duo with the result
            iterations += 1;
        } else {
            break;
        }
        
        div_shifted >>= 1;
        if div_shifted == 0 {
            break;
        }
    }
    
    if iterations == MAX_ITER {
        0xdeadbeef  // Infinite loop detected
    } else {
        iterations
    }
}

#[inline(always)]
fn u32_normalization_shift(duo: u32, div: u32, full_normalization: bool) -> usize {
    let mut shl = (div.leading_zeros() - duo.leading_zeros()) as usize;
    if full_normalization {
        if duo < (div << shl) {
            shl -= 1;
        }
    }
    shl
}

// run: divide64(0, 0, 100) == 0
// run: divide64(0, 100, 2) == 1
// run: divide64(0, 16, 2) == 1

