#![no_std]

/// Test 08: Use exact values from the failing test case
/// Tests with the specific input that causes the infinite loop
#[no_mangle]
pub fn divide64(dividend_hi: u32, dividend_lo: u32, divisor: u32) -> u32 {
    // This matches the failing test: divide64(0, 0, 100)
    // duo = 0, div = 100
    let duo = ((dividend_hi as u64) << 32) | (dividend_lo as u64);
    let div = divisor as u64;
    
    // Calculate normalization shift
    let duo_hi = (duo >> 32) as u32;
    let div_lo = div as u32;
    let norm_shift = div_lo.leading_zeros() - duo_hi.leading_zeros();
    let n = 32;
    let shl = if norm_shift == 0 { n - 1 } else { n - norm_shift as usize };
    
    let mut div_shifted = div << shl;
    let mut duo = duo;
    let mut iterations = 0;
    const MAX_ITER: u32 = 32;
    
    loop {
        if iterations >= MAX_ITER {
            return 0xdeadbeef; // Infinite loop detected
        }
        iterations += 1;
        
        let sub = duo.wrapping_sub(div_shifted);
        if 0 <= (sub as i64) {
            duo = sub;
            let duo_hi_check = (duo >> 32) as u32;
            if duo_hi_check == 0 {
                return (duo as u32) / div_lo;
            }
        }
        div_shifted >>= 1;
        if div_shifted == 0 {
            break;
        }
    }
    
    iterations
}

// run: divide64(0, 0, 100) == 0
// run: divide64(0, 100, 2) == 50
// run: divide64(0, 16, 2) == 8

