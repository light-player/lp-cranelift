//! Value parsing and comparison operations.

use crate::file_update::format_glsl_value;
use crate::filetest::ComparisonOp;
use anyhow::Result;
use lp_glsl::GlslValue;

/// Parse a GLSL value from a string.
/// Supports scalars, vectors, and matrices.
pub fn parse_glsl_value(s: &str) -> Result<GlslValue> {
    let s = s.trim();

    // Check for uint suffix (u or U)
    if s.ends_with('u') || s.ends_with('U') {
        let num_str = &s[..s.len() - 1];
        if let Ok(u) = num_str.parse::<u32>() {
            return Ok(GlslValue::U32(u));
        }
    }

    // Try parsing as integer
    if let Ok(i) = s.parse::<i32>() {
        return Ok(GlslValue::I32(i));
    }

    // Try parsing as float
    if let Ok(f) = s.parse::<f32>() {
        return Ok(GlslValue::F32(f));
    }

    // Try parsing as boolean
    match s {
        "true" => return Ok(GlslValue::Bool(true)),
        "false" => return Ok(GlslValue::Bool(false)),
        _ => {}
    }

    // Try parsing as vector or matrix constructor using GlslValue::parse
    // This uses the GLSL parser to handle constructors like vec2(1.0, 2.0)
    if let Ok(value) = GlslValue::parse(s) {
        return Ok(value);
    }

    anyhow::bail!("failed to parse GLSL value: {}", s)
}

/// Compare actual and expected values.
pub fn compare_results(
    actual: &GlslValue,
    expected: &GlslValue,
    comparison: ComparisonOp,
    tolerance: Option<f32>,
) -> Result<(), String> {
    match comparison {
        ComparisonOp::Exact => {
            if actual.eq(expected) {
                Ok(())
            } else {
                Err(format!(
                    "expected {}, got {}",
                    format_glsl_value(expected),
                    format_glsl_value(actual)
                ))
            }
        }
        ComparisonOp::Approx => {
            let tolerance = tolerance.unwrap_or(GlslValue::DEFAULT_TOLERANCE);
            if actual.approx_eq(expected, tolerance) {
                Ok(())
            } else {
                Err(format!(
                    "expected {} (tolerance: {}), got {}",
                    format_glsl_value(expected),
                    tolerance,
                    format_glsl_value(actual)
                ))
            }
        }
    }
}
