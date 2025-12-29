//! Parse `// run:` directives from source text.

extern crate alloc;
use crate::test_util::expectations::{ComparisonOp, RunDirective};
use crate::test_util::number::{parse_number, ParseError};
use alloc::{format, string::{String, ToString}, vec::Vec};

/// Result type for parsing.
pub type Result<T> = core::result::Result<T, ParseError>;

/// Extract and parse all `// run:` directives from source text.
pub fn parse_run_directives(source: &str) -> Result<Vec<RunDirective>> {
    let mut directives = Vec::new();

    for (line_num, line) in source.lines().enumerate() {
        let line_number = line_num + 1; // 1-indexed
        let trimmed = line.trim();

        // Check if this is a run directive
        if let Some(run_content) = trimmed
            .strip_prefix("// run:")
            .or_else(|| trimmed.strip_prefix("// #run:"))
        {
            let directive = parse_run_directive(run_content.trim(), line_number)?;
            directives.push(directive);
        }
    }

    Ok(directives)
}

/// Parse a single `// run:` line content into a `RunDirective`.
///
/// Format: `<expression> == <expected>` or `<expression> ~= <expected> [ (tolerance: <value>) ]`
/// Expression format: `%function_name(arg1, arg2, ...)`
fn parse_run_directive(line: &str, line_number: usize) -> Result<RunDirective> {
    // Parse comparison operator and split expression from expected
    let (comparison, expr, expected_with_tolerance) = if let Some(pos) = line.rfind(" == ") {
        let expr = line[..pos].trim();
        let expected = line[pos + 4..].trim();
        (ComparisonOp::Exact, expr, expected)
    } else if let Some(pos) = line.rfind(" ~= ") {
        let expr = line[..pos].trim();
        let expected = line[pos + 4..].trim();
        (ComparisonOp::Approx { tolerance: ComparisonOp::DEFAULT_TOLERANCE }, expr, expected)
    } else {
        return Err(ParseError::new(&format!(
            "invalid run directive format at line {}: expected '==' or '~='",
            line_number
        )));
    };

    // Parse tolerance if present: (tolerance: 0.001)
    let (expected_str, comparison) = if let Some(tolerance_start) = expected_with_tolerance.find("(tolerance:") {
        let expected = expected_with_tolerance[..tolerance_start].trim();
        let tolerance_str = expected_with_tolerance[tolerance_start..]
            .strip_prefix("(tolerance:")
            .and_then(|s| s.strip_suffix(")"))
            .map(|s| s.trim());
        
        let tolerance = if let Some(tol_str) = tolerance_str {
            tol_str.parse::<f32>().map_err(|_| {
                ParseError::new(&format!("invalid tolerance value at line {}", line_number))
            })?
        } else {
            ComparisonOp::DEFAULT_TOLERANCE
        };
        
        (expected, ComparisonOp::Approx { tolerance })
    } else {
        (expected_with_tolerance, comparison)
    };

    // Parse function call expression: %function_name(arg1, arg2, ...)
    let (function_name, arguments, arg_formats) = parse_function_call(expr, line_number)?;

    // Parse expected value
    let (expected, expected_format) = parse_number(expected_str)
        .map_err(|e| ParseError::new(&format!("failed to parse expected value at line {}: {}", line_number, e)))?;

    Ok(RunDirective {
        function_name,
        arguments,
        arg_formats,
        expected,
        expected_format,
        comparison,
        line_number,
    })
}

/// Parse a function call expression: `%function_name(arg1, arg2, ...)`
fn parse_function_call(expr: &str, line_number: usize) -> Result<(String, Vec<crate::test_util::number::TestNum>, Vec<crate::test_util::number::NumFormat>)> {
    let expr = expr.trim();

    // Must start with %
    if !expr.starts_with('%') {
        return Err(ParseError::new(&format!(
            "invalid function call expression at line {}: must start with '%'",
            line_number
        )));
    }

    // Find opening parenthesis
    let paren_pos = expr
        .find('(')
        .ok_or_else(|| ParseError::new(&format!("invalid function call expression at line {}: missing '('", line_number)))?;

    // Extract function name (skip %)
    let function_name = expr[1..paren_pos].trim().to_string();
    if function_name.is_empty() {
        return Err(ParseError::new(&format!(
            "invalid function call expression at line {}: empty function name",
            line_number
        )));
    }

    // Find closing parenthesis
    let closing_paren = expr
        .rfind(')')
        .ok_or_else(|| ParseError::new(&format!("invalid function call expression at line {}: missing ')'", line_number)))?;

    // Extract arguments string
    let args_str = &expr[paren_pos + 1..closing_paren].trim();

    // Parse arguments
    let (arguments, arg_formats): (Vec<_>, Vec<_>) = if args_str.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        args_str
            .split(',')
            .map(|arg| {
                parse_number(arg.trim()).map_err(|e| {
                    ParseError::new(&format!("failed to parse argument '{}' at line {}: {}", arg, line_number, e))
                })
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .unzip()
    };

    Ok((function_name, arguments, arg_formats))
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use crate::test_util::number::{NumFormat, TestNum};
    use alloc::format;

    #[test]
    fn test_parse_simple_exact() {
        let source = "// run: %test_func(0x00040000) == 0x00020000";
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 1);

        let dir = &directives[0];
        assert_eq!(dir.function_name, "test_func");
        assert_eq!(dir.arguments.len(), 1);
        assert_eq!(dir.arg_formats.len(), 1);
        assert!(matches!(dir.comparison, ComparisonOp::Exact));
        assert_eq!(dir.line_number, 1);
    }

    #[test]
    fn test_parse_approximate() {
        let source = "// run: %test_func(1.0fx32) ~= 1.0fx32";
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 1);

        let dir = &directives[0];
        assert!(matches!(dir.comparison, ComparisonOp::Approx { tolerance: 0.000001 }));
    }

    #[test]
    fn test_parse_with_tolerance() {
        let source = "// run: %test_func(1.0fx32) ~= 1.0fx32 (tolerance: 0.001)";
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 1);

        let dir = &directives[0];
        assert!(matches!(dir.comparison, ComparisonOp::Approx { tolerance: 0.001 }));
    }

    #[test]
    fn test_parse_multiple_args() {
        let source = "// run: %test_func(1, 2, 3) == 6";
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 1);

        let dir = &directives[0];
        assert_eq!(dir.arguments.len(), 3);
    }

    #[test]
    fn test_parse_multiple_directives() {
        let source = r#"
// run: %test_func(1) == 1
// run: %test_func(2) == 4
// run: %test_func(3) == 9
"#;
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 3);
    }

    #[test]
    fn test_parse_different_formats() {
        let source = "// run: %test_func(0x00040000, 4.0fx32, 65536, 1.0f32) == 0";
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 1);

        let dir = &directives[0];
        assert_eq!(dir.arguments.len(), 4);
        assert_eq!(dir.arg_formats.len(), 4);
        
        // Check formats are preserved
        assert_eq!(dir.arg_formats[0], NumFormat::Hex);
        assert_eq!(dir.arg_formats[1], NumFormat::Fixed32);
        assert_eq!(dir.arg_formats[2], NumFormat::Decimal);
        assert_eq!(dir.arg_formats[3], NumFormat::Float32);
    }

    #[test]
    fn test_parse_no_args() {
        let source = "// run: %test_func() == 42";
        let directives = parse_run_directives(source).unwrap();
        assert_eq!(directives.len(), 1);

        let dir = &directives[0];
        assert_eq!(dir.arguments.len(), 0);
    }

    #[test]
    fn test_parse_errors() {
        // Missing comparison operator
        assert!(parse_run_directives("// run: %test_func(1) 42").is_err());

        // Missing function name
        assert!(parse_run_directives("// run: %(1) == 42").is_err());

        // Missing parentheses
        assert!(parse_run_directives("// run: %test_func 1 == 42").is_err());

        // Invalid argument
        assert!(parse_run_directives("// run: %test_func(invalid) == 42").is_err());
    }
}

