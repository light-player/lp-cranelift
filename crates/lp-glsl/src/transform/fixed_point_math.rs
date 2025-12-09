//! Fixed-point math function implementations using CORDIC algorithm
//!
//! This module provides production-quality implementations of trigonometric
//! and other math functions using pure integer arithmetic via the CORDIC
//! (COordinate Rotation DIgital Computer) algorithm.

use crate::error::GlslError;
use crate::transform::fixed_point::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::ir::{
    Value, InstBuilder, condcodes::IntCC,
};
use cranelift_codegen::cursor::FuncCursor;

/// Generate precomputed CORDIC angle table for a given format
///
/// Returns angles atan(2^-i) for i=0..N-1 in fixed-point format
fn generate_angle_table(format: FixedPointFormat) -> Vec<i64> {
    let iterations = match format {
        FixedPointFormat::Fixed16x16 => 16,
        FixedPointFormat::Fixed32x32 => 32,
    };
    
    let mut angles = Vec::with_capacity(iterations);
    let scale = match format {
        FixedPointFormat::Fixed16x16 => 65536.0,
        FixedPointFormat::Fixed32x32 => 4294967296.0,
    };
    
    for i in 0..iterations {
        let angle_rad = (2.0_f64).powi(-(i as i32)).atan();
        let fixed_angle = (angle_rad * scale) as i64;
        angles.push(fixed_angle);
    }
    
    angles
}

/// Generate CORDIC gain factor K in fixed-point format
///
/// K = ∏(1/√(1 + 2^(-2i))) for i=0..N-1
fn generate_gain_factor(format: FixedPointFormat) -> i64 {
    let iterations = match format {
        FixedPointFormat::Fixed16x16 => 16,
        FixedPointFormat::Fixed32x32 => 32,
    };
    
    let mut k = 1.0;
    for i in 0..iterations {
        k *= 1.0 / (1.0 + 2.0_f64.powi(-2 * i as i32)).sqrt();
    }
    
    let scale = match format {
        FixedPointFormat::Fixed16x16 => 65536.0,
        FixedPointFormat::Fixed32x32 => 4294967296.0,
    };
    
    (k * scale) as i64
}

/// Generate CORDIC gain factor inverse (1/K) in fixed-point format
fn generate_gain_factor_inverse(format: FixedPointFormat) -> i64 {
    let iterations = match format {
        FixedPointFormat::Fixed16x16 => 16,
        FixedPointFormat::Fixed32x32 => 32,
    };
    
    let mut k = 1.0;
    for i in 0..iterations {
        k *= 1.0 / (1.0 + 2.0_f64.powi(-2 * i as i32)).sqrt();
    }
    
    let scale = match format {
        FixedPointFormat::Fixed16x16 => 65536.0,
        FixedPointFormat::Fixed32x32 => 4294967296.0,
    };
    
    ((1.0 / k) * scale) as i64
}

/// Generate π constant in fixed-point format
fn generate_pi(format: FixedPointFormat) -> i64 {
    let scale = match format {
        FixedPointFormat::Fixed16x16 => 65536.0,
        FixedPointFormat::Fixed32x32 => 4294967296.0,
    };
    (std::f64::consts::PI * scale) as i64
}

/// Generate π/2 constant in fixed-point format
fn generate_pi_over_2(format: FixedPointFormat) -> i64 {
    let scale = match format {
        FixedPointFormat::Fixed16x16 => 65536.0,
        FixedPointFormat::Fixed32x32 => 4294967296.0,
    };
    (std::f64::consts::PI / 2.0 * scale) as i64
}

/// Generate 2π constant in fixed-point format
fn generate_2pi(format: FixedPointFormat) -> i64 {
    let scale = match format {
        FixedPointFormat::Fixed16x16 => 65536.0,
        FixedPointFormat::Fixed32x32 => 4294967296.0,
    };
    (2.0 * std::f64::consts::PI * scale) as i64
}

/// CORDIC rotation mode: compute sin and cos simultaneously
///
/// Input: angle in fixed-point format (radians)
/// Output: (sin, cos) tuple in fixed-point format
fn cordic_rotation_mode(
    cursor: &mut FuncCursor,
    angle: Value,
    format: FixedPointFormat,
) -> Result<(Value, Value), GlslError> {
    let target_type = format.cranelift_type();
    let iterations = match format {
        FixedPointFormat::Fixed16x16 => 16,
        FixedPointFormat::Fixed32x32 => 32,
    };
    
    // Generate angle table and gain factor
    let angle_table = generate_angle_table(format);
    let gain_k = generate_gain_factor(format);
    
    // Initialize: x = K, y = 0, z = angle
    let mut x = cursor.ins().iconst(target_type, gain_k);
    let mut y = cursor.ins().iconst(target_type, 0);
    let mut z = angle;
    
    // CORDIC iterations
    for i in 0..iterations {
        // Determine rotation direction: d = sign(z)
        let zero = cursor.ins().iconst(target_type, 0);
        let z_lt_zero = cursor.ins().icmp(IntCC::SignedLessThan, z, zero);
        let d = cursor.ins().select(z_lt_zero, 
            cursor.ins().iconst(target_type, -1),
            cursor.ins().iconst(target_type, 1));
        
        // Shift amount: 2^-i = right shift by i
        let shift_amount_val = cursor.ins().iconst(target_type, i as i64);
        
        // x_new = x - d * (y >> i)
        let y_shifted = cursor.ins().sshr(y, shift_amount_val);
        let d_times_y = cursor.ins().imul(d, y_shifted);
        let x_new = cursor.ins().isub(x, d_times_y);
        
        // y_new = y + d * (x >> i)
        let x_shifted = cursor.ins().sshr(x, shift_amount_val);
        let d_times_x = cursor.ins().imul(d, x_shifted);
        let y_new = cursor.ins().iadd(y, d_times_x);
        
        // z_new = z - d * angle_table[i]
        let angle_const = cursor.ins().iconst(target_type, angle_table[i]);
        let d_times_angle = cursor.ins().imul(d, angle_const);
        let z_new = cursor.ins().isub(z, d_times_angle);
        
        x = x_new;
        y = y_new;
        z = z_new;
    }
    
    // Results: sin = y, cos = x (gain already included in initial x)
    Ok((y, x))
}

/// Reduce angle to [0, π/2] range and return quadrant info
///
/// Returns: (reduced_angle, quadrant) where quadrant indicates:
/// - 0: [0, π/2] - no transformation needed
/// - 1: [π/2, π] - use sin(π - x), -cos(π - x)
/// - 2: [π, 3π/2] - use -sin(x - π), -cos(x - π)
/// - 3: [3π/2, 2π] - use -sin(2π - x), cos(2π - x)
fn reduce_angle_to_quadrant(
    cursor: &mut FuncCursor,
    angle: Value,
    format: FixedPointFormat,
) -> Result<(Value, Value), GlslError> {
    let target_type = format.cranelift_type();
    let pi = generate_pi(format);
    let pi_over_2 = generate_pi_over_2(format);
    let pi_2 = generate_2pi(format);
    
    let pi_const = cursor.ins().iconst(target_type, pi);
    let pi_over_2_const = cursor.ins().iconst(target_type, pi_over_2);
    let pi_2_const = cursor.ins().iconst(target_type, pi_2);
    let zero = cursor.ins().iconst(target_type, 0);
    
    // Reduce to [0, 2π)
    // angle = angle % (2π)
    // For now, assume angle is already in reasonable range
    // TODO: Implement proper modulo for large angles
    
    // Compute intermediate values first to avoid borrow conflicts
    let pi_plus_pi_over_2 = cursor.ins().iadd(pi_const, pi_over_2_const);
    let one = cursor.ins().iconst(target_type, 1);
    let two = cursor.ins().iconst(target_type, 2);
    let three = cursor.ins().iconst(target_type, 3);
    
    // Determine quadrant
    let in_q1 = cursor.ins().icmp(IntCC::SignedLessThanOrEqual, angle, pi_over_2_const);
    let in_q2 = cursor.ins().icmp(IntCC::SignedLessThanOrEqual, angle, pi_const);
    let in_q3 = cursor.ins().icmp(IntCC::SignedLessThanOrEqual, angle, pi_plus_pi_over_2);
    let in_q4 = cursor.ins().icmp(IntCC::SignedGreaterThan, angle, pi_plus_pi_over_2);
    
    // Compute reduced angles for each quadrant
    let pi_minus_angle = cursor.ins().isub(pi_const, angle);
    let angle_minus_pi = cursor.ins().isub(angle, pi_const);
    let pi_2_minus_angle = cursor.ins().isub(pi_2_const, angle);
    
    // Select reduced angle based on quadrant
    let reduced_q1 = cursor.ins().select(in_q2, pi_minus_angle, angle);
    let reduced_q2 = cursor.ins().select(in_q3, angle_minus_pi, reduced_q1);
    let reduced = cursor.ins().select(in_q4, pi_2_minus_angle, reduced_q2);
    
    // Determine quadrant number (0-3)
    let not_q1 = cursor.ins().bnot(in_q1);
    let q1_cond = cursor.ins().band(in_q2, not_q1);
    let q0 = cursor.ins().select(in_q1, zero, zero);
    let q1 = cursor.ins().select(q1_cond, one, q0);
    
    let not_q2 = cursor.ins().bnot(in_q2);
    let q2_cond = cursor.ins().band(in_q3, not_q2);
    let q2 = cursor.ins().select(q2_cond, two, q1);
    
    let q3 = cursor.ins().select(in_q4, three, q2);
    
    Ok((reduced, q3))
}

/// Generate inline IR for sin(x) where x is fixed-point
pub fn generate_sin_fixed(
    cursor: &mut FuncCursor,
    x: Value,
    format: FixedPointFormat,
) -> Result<Value, GlslError> {
    // Reduce angle to [0, π/2] and get quadrant
    let (reduced_angle, quadrant) = reduce_angle_to_quadrant(cursor, x, format)?;
    
    // Compute sin/cos of reduced angle using CORDIC
    let (sin_val, _cos_val) = cordic_rotation_mode(cursor, reduced_angle, format)?;
    
    // Apply quadrant transformations
    let target_type = format.cranelift_type();
    let zero = cursor.ins().iconst(target_type, 0);
    let one = cursor.ins().iconst(target_type, 1);
    let two = cursor.ins().iconst(target_type, 2);
    let three = cursor.ins().iconst(target_type, 3);
    
    let q0 = cursor.ins().icmp(IntCC::Equal, quadrant, zero);
    let q1 = cursor.ins().icmp(IntCC::Equal, quadrant, one);
    let q2 = cursor.ins().icmp(IntCC::Equal, quadrant, two);
    let q3 = cursor.ins().icmp(IntCC::Equal, quadrant, three);
    
    // Q0: sin(x) = sin(reduced)
    // Q1: sin(x) = sin(π - reduced) = sin(reduced)
    // Q2: sin(x) = -sin(reduced - π) = -sin(reduced)
    // Q3: sin(x) = -sin(2π - reduced) = -sin(reduced)
    let neg_sin = cursor.ins().ineg(sin_val);
    let result_q2 = cursor.ins().select(q2, neg_sin, sin_val);
    let result_q3 = cursor.ins().select(q3, neg_sin, result_q2);
    let result_q1 = cursor.ins().select(q1, sin_val, result_q3);
    let result = cursor.ins().select(q0, sin_val, result_q1);
    
    Ok(result)
}

/// Generate inline IR for cos(x) where x is fixed-point
pub fn generate_cos_fixed(
    cursor: &mut FuncCursor,
    x: Value,
    format: FixedPointFormat,
) -> Result<Value, GlslError> {
    // Reduce angle to [0, π/2] and get quadrant
    let (reduced_angle, quadrant) = reduce_angle_to_quadrant(cursor, x, format)?;
    
    // Compute sin/cos of reduced angle using CORDIC
    let (_sin_val, cos_val) = cordic_rotation_mode(cursor, reduced_angle, format)?;
    
    // Apply quadrant transformations
    let target_type = format.cranelift_type();
    let zero = cursor.ins().iconst(target_type, 0);
    let one = cursor.ins().iconst(target_type, 1);
    let two = cursor.ins().iconst(target_type, 2);
    let three = cursor.ins().iconst(target_type, 3);
    
    let q0 = cursor.ins().icmp(IntCC::Equal, quadrant, zero);
    let q1 = cursor.ins().icmp(IntCC::Equal, quadrant, one);
    let q2 = cursor.ins().icmp(IntCC::Equal, quadrant, two);
    let q3 = cursor.ins().icmp(IntCC::Equal, quadrant, three);
    
    // Q0: cos(x) = cos(reduced)
    // Q1: cos(x) = -cos(π - reduced) = -cos(reduced)
    // Q2: cos(x) = -cos(reduced - π) = -cos(reduced)
    // Q3: cos(x) = cos(2π - reduced) = cos(reduced)
    let neg_cos = cursor.ins().ineg(cos_val);
    let result_q1 = cursor.ins().select(q1, neg_cos, cos_val);
    let result_q2 = cursor.ins().select(q2, neg_cos, result_q1);
    let result_q3 = cursor.ins().select(q3, cos_val, result_q2);
    let result = cursor.ins().select(q0, cos_val, result_q3);
    
    Ok(result)
}

