//! Square root implementation for fixed16x16 using reciprocal multiplication.
//!
//! This module provides square root for fixed16x16 format using Newton-Raphson
//! method with reciprocal multiplication to avoid i64 division.

const SHIFT: u32 = 16;

/// Divide x_scaled (i64) by guess using reciprocal multiplication.
///
/// Algorithm:
/// 1. Truncate guess to i32 and compute its reciprocal
/// 2. Multiply x_scaled by reciprocal and scale appropriately
/// 3. Result approximates x_scaled / guess
///
/// Note: Both x_scaled and guess are in the scaled space (i64),
/// and the result is also in the scaled space.
fn divide_by_reciprocal(x_scaled: i64, guess: i64) -> i64 {
    // Ensure guess is positive and non-zero for reciprocal
    let guess_abs = guess.abs();
    let guess_safe = if guess_abs == 0 { 1 } else { guess_abs };
    
    // Truncate to i32 for reciprocal calculation (we lose some precision here)
    let guess_i32 = guess_safe.min(i32::MAX as i64) as i32;
    let guess_safe_u32 = guess_i32.abs() as u32;
    let guess_safe_u32 = if guess_safe_u32 == 0 { 1 } else { guess_safe_u32 };
    
    // Compute reciprocal: 0x8000_0000 / guess_safe (i32 division)
    // This represents 1/guess scaled by 2^31
    let recip = 0x8000_0000u32 / guess_safe_u32;
    
    // Multiply x_scaled by reciprocal
    // Formula: (x_scaled * recip * 2) >> SHIFT
    // This approximates x_scaled / guess
    let x_scaled_abs = x_scaled.abs() as u64;
    let recip_u64 = recip as u64;
    
    // Multiply: x_scaled * recip * 2
    // Check for potential overflow - if x_scaled is very large, we need to be careful
    // For safety, we can do the multiplication in steps or use saturating arithmetic
    let mul_result = x_scaled_abs.saturating_mul(recip_u64).saturating_mul(2u64);
    
    // Right shift by SHIFT (16) to account for fixed-point scaling
    let quotient = (mul_result >> SHIFT) as i64;
    
    // Apply sign: if x_scaled and guess have different signs, negate
    let result_sign = if (x_scaled < 0) != (guess < 0) { -1 } else { 1 };
    quotient * result_sign
}

/// Compute square root using Newton-Raphson with reciprocal multiplication.
///
/// Algorithm:
/// 1. Scale input: x_scaled = x_fixed << 16 (i64)
/// 2. Initial guess: max(x_scaled >> 9, 1)
/// 3. Iterate 6 times: guess = (guess + x_scaled / guess) >> 1
///    where x_scaled / guess is computed using reciprocal multiplication
/// 4. Result: guess >> 8 (truncate to i32)
pub fn fixed32_sqrt(x_fixed: i32) -> i32 {
    // Handle edge cases
    if x_fixed <= 0 {
        return 0;
    }
    
    // Convert to i64 and scale up for better precision
    // x_scaled = x_fixed << 16 = x_fixed * 65536
    let x_scaled = (x_fixed as i64) << SHIFT;
    
    // Initial guess for sqrt(x_scaled) using a better approximation
    // After testing, >> 9 works as a compromise
    let mut guess = (x_scaled >> 9).max(1);
    
    // Newton-Raphson iterations: guess = (guess + x_scaled / guess) >> 1
    // We use 6 iterations for better precision, especially for larger values
    for _ in 0..6 {
        // Compute x_scaled / guess using reciprocal multiplication
        let x_div_guess = divide_by_reciprocal(x_scaled, guess);
        
        // Newton-Raphson step: guess_new = (guess + x_scaled / guess) >> 1
        let sum = guess + x_div_guess;
        guess = sum >> 1;
        
        // Ensure guess doesn't become zero
        if guess == 0 {
            guess = 1;
        }
    }
    
    // Based on the test output, guess is 256 times too large, so:
    (guess >> 8) as i32
}