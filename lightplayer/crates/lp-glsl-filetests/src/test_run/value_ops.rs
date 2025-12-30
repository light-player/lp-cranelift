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

/// Parse a function call expression (e.g., "add_float(1.5, 2.5)") into function name and arguments.
/// Returns (function_name, argument_strings).
pub fn parse_function_call(expression: &str) -> Result<(String, Vec<String>)> {
    let expression = expression.trim();

    // Find the opening parenthesis
    let open_paren = expression
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("function call must contain '(': {}", expression))?;

    // Extract function name (everything before the opening parenthesis)
    let func_name = expression[..open_paren].trim().to_string();
    if func_name.is_empty() {
        return Err(anyhow::anyhow!(
            "function name is empty in expression: {}",
            expression
        ));
    }

    // Find the matching closing parenthesis
    let args_str = &expression[open_paren + 1..];
    let mut paren_depth = 1;
    let mut close_paren_pos = None;

    for (i, ch) in args_str.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth -= 1;
                if paren_depth == 0 {
                    close_paren_pos = Some(i);
                    break;
                }
            }
            _ => {}
        }
    }

    let close_paren_pos = close_paren_pos
        .ok_or_else(|| anyhow::anyhow!("unmatched parentheses in expression: {}", expression))?;

    // Extract arguments string (between parentheses)
    let args_str = &args_str[..close_paren_pos];

    // Parse arguments (split by comma, respecting nested parentheses)
    let mut args = Vec::new();
    if !args_str.trim().is_empty() {
        let mut current_arg = String::new();
        let mut paren_depth = 0;

        for ch in args_str.chars() {
            match ch {
                '(' => {
                    paren_depth += 1;
                    current_arg.push(ch);
                }
                ')' => {
                    paren_depth -= 1;
                    current_arg.push(ch);
                }
                ',' => {
                    if paren_depth == 0 {
                        // This comma is at the top level, split here
                        args.push(current_arg.trim().to_string());
                        current_arg.clear();
                    } else {
                        // This comma is inside nested parentheses, keep it
                        current_arg.push(ch);
                    }
                }
                _ => current_arg.push(ch),
            }
        }

        // Add the last argument
        if !current_arg.trim().is_empty() {
            args.push(current_arg.trim().to_string());
        }
    }

    Ok((func_name, args))
}

/// Parse function call arguments from strings to GlslValue.
pub fn parse_function_arguments(arg_strings: &[String]) -> Result<Vec<GlslValue>> {
    arg_strings
        .iter()
        .map(|arg_str| parse_glsl_value(arg_str))
        .collect()
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
