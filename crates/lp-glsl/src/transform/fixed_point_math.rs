//! Fixed-point math function implementations using CORDIC algorithm
//!
//! This module provides production-quality implementations of trigonometric
//! and other math functions using pure integer arithmetic via the CORDIC
//! (COordinate Rotation DIgital Computer) algorithm.
//!
//! Note: Math functions (sin, cos) are now handled by intrinsic implementations
//! in the compiler. This module only contains tanh for now.

// Keep the old tanh implementation here for now (not yet moved)
use crate::error::GlslError;
use crate::transform::fixed_point::FixedPointFormat;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

use cranelift_codegen::cursor::FuncCursor;
use cranelift_codegen::ir::{InstBuilder, Value, condcodes::IntCC};

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

    // Pre-create constants used in loop
    let zero = cursor.ins().iconst(target_type, 0);
    let minus_one = cursor.ins().iconst(target_type, -1);
    let one = cursor.ins().iconst(target_type, 1);

    // CORDIC iterations
    for i in 0..iterations {
        // Determine rotation direction: d = sign(z)
        let z_lt_zero = cursor.ins().icmp(IntCC::SignedLessThan, z, zero);
        let d = cursor.ins().select(z_lt_zero, minus_one, one);

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

    // After CORDIC: x = K*cos(angle), y = K*sin(angle)
    // Need to divide by K to get sin and cos
    // Use multiply by 1/K for efficiency: result = (value * (1/K)) >> shift_amount
    let inv_k = generate_gain_factor_inverse(format);
    let shift_amount = format.shift_amount();

    match format {
        FixedPointFormat::Fixed16x16 => {
            // For 16.16: sin = (y * (1/K)) >> 16
            let inv_k_const = cursor
                .ins()
                .iconst(cranelift_codegen::ir::types::I32, inv_k);
            let y_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, y);
            let inv_k_ext = cursor
                .ins()
                .sextend(cranelift_codegen::ir::types::I64, inv_k_const);
            let mul_y = cursor.ins().imul(y_ext, inv_k_ext);
            let shift_const = cursor
                .ins()
                .iconst(cranelift_codegen::ir::types::I64, shift_amount);
            let sin_result = cursor.ins().sshr(mul_y, shift_const);
            let sin = cursor
                .ins()
                .ireduce(cranelift_codegen::ir::types::I32, sin_result);

            // cos = (x * (1/K)) >> 16
            let x_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, x);
            let mul_x = cursor.ins().imul(x_ext, inv_k_ext);
            let cos_result = cursor.ins().sshr(mul_x, shift_const);
            let cos = cursor
                .ins()
                .ireduce(cranelift_codegen::ir::types::I32, cos_result);

            Ok((sin, cos))
        }
        FixedPointFormat::Fixed32x32 => {
            // For 32.32: sin = (y * (1/K)) >> 32
            let inv_k_const = cursor
                .ins()
                .iconst(cranelift_codegen::ir::types::I64, inv_k);
            let y_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, y);
            let inv_k_ext = cursor
                .ins()
                .sextend(cranelift_codegen::ir::types::I128, inv_k_const);
            let mul_y = cursor.ins().imul(y_ext, inv_k_ext);
            let shift_const = cursor
                .ins()
                .iconst(cranelift_codegen::ir::types::I64, shift_amount);
            let sin_result = cursor.ins().sshr(mul_y, shift_const);
            let sin = cursor
                .ins()
                .ireduce(cranelift_codegen::ir::types::I64, sin_result);

            // cos = (x * (1/K)) >> 32
            let x_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, x);
            let mul_x = cursor.ins().imul(x_ext, inv_k_ext);
            let cos_result = cursor.ins().sshr(mul_x, shift_const);
            let cos = cursor
                .ins()
                .ireduce(cranelift_codegen::ir::types::I64, cos_result);

            Ok((sin, cos))
        }
    }
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
    let in_q1 = cursor
        .ins()
        .icmp(IntCC::SignedLessThanOrEqual, angle, pi_over_2_const);
    let in_q2 = cursor
        .ins()
        .icmp(IntCC::SignedLessThanOrEqual, angle, pi_const);
    let in_q3 = cursor
        .ins()
        .icmp(IntCC::SignedLessThanOrEqual, angle, pi_plus_pi_over_2);
    let in_q4 = cursor
        .ins()
        .icmp(IntCC::SignedGreaterThan, angle, pi_plus_pi_over_2);

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

// generate_sin_fixed and generate_cos_fixed are now in fixed_point::fixed::trig
// and re-exported above

/// Generate inline IR for tanh(x) where x is fixed-point
///
/// Uses approximation: tanh(x) ≈ x / (1 + |x|) for small values
/// For larger values, clamps to ±1
pub fn generate_tanh_fixed(
    cursor: &mut FuncCursor,
    x: Value,
    format: FixedPointFormat,
) -> Result<Value, GlslError> {
    let target_type = format.cranelift_type();
    let zero = cursor.ins().iconst(target_type, 0);
    let one = cursor.ins().iconst(target_type, 1);

    // Get absolute value and sign
    let is_negative = cursor.ins().icmp(IntCC::SignedLessThan, x, zero);
    let abs_x = cursor.ins().iabs(x);

    // For small values: tanh(x) ≈ x / (1 + |x|)
    // This works well for |x| < 1 in fixed-point representation
    // For larger values, we clamp to ±1

    // Compute 1 + |x|
    let one_plus_abs = cursor.ins().iadd(one, abs_x);

    // Compute x / (1 + |x|) using fixed-point division
    let shift_amount = format.shift_amount();
    let result = match format {
        FixedPointFormat::Fixed16x16 => {
            // x_ext = x extended to I64
            let x_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I64, x);
            let shift_const = cursor
                .ins()
                .iconst(cranelift_codegen::ir::types::I64, shift_amount);
            let x_shifted = cursor.ins().ishl(x_ext, shift_const);
            let one_plus_abs_ext = cursor
                .ins()
                .sextend(cranelift_codegen::ir::types::I64, one_plus_abs);
            let div_result = cursor.ins().sdiv(x_shifted, one_plus_abs_ext);
            cursor
                .ins()
                .ireduce(cranelift_codegen::ir::types::I32, div_result)
        }
        FixedPointFormat::Fixed32x32 => {
            // x_ext = x extended to I128
            let x_ext = cursor.ins().sextend(cranelift_codegen::ir::types::I128, x);
            let shift_const = cursor
                .ins()
                .iconst(cranelift_codegen::ir::types::I64, shift_amount);
            let x_shifted = cursor.ins().ishl(x_ext, shift_const);
            let one_plus_abs_ext = cursor
                .ins()
                .sextend(cranelift_codegen::ir::types::I128, one_plus_abs);
            let div_result = cursor.ins().sdiv(x_shifted, one_plus_abs_ext);
            cursor
                .ins()
                .ireduce(cranelift_codegen::ir::types::I64, div_result)
        }
    };

    // For very large values, clamp to ±1
    // If |x| > threshold (e.g., 2 in fixed-point), return sign(x)
    let threshold = match format {
        FixedPointFormat::Fixed16x16 => 2 << 16,     // 2.0 in 16.16
        FixedPointFormat::Fixed32x32 => 2_i64 << 32, // 2.0 in 32.32
    };
    let threshold_val = cursor.ins().iconst(target_type, threshold);
    let is_large = cursor
        .ins()
        .icmp(IntCC::SignedGreaterThan, abs_x, threshold_val);

    // Clamp result: if large, use sign(x), otherwise use computed result
    let neg_one = cursor.ins().ineg(one);
    let sign_result = cursor.ins().select(is_negative, neg_one, one);
    let final_result = cursor.ins().select(is_large, sign_result, result);

    Ok(final_result)
}
