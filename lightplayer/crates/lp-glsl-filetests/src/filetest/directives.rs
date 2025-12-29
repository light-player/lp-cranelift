//! Directive parsing logic.

use crate::filetest::{ComparisonOp, RunDirective, TestType};
use anyhow::Result;

/// Parse test type from a line.
pub fn parse_test_type(line: &str) -> Option<TestType> {
    let trimmed = line.trim();
    if trimmed == "// test compile" {
        Some(TestType::Compile)
    } else if trimmed == "// test transform.fixed32" {
        Some(TestType::TransformFixed32)
    } else if trimmed.starts_with("// test run") {
        Some(TestType::Run)
    } else {
        None
    }
}

/// Parse target directive from a line.
pub fn parse_target_directive(line: &str) -> Option<String> {
    line.trim()
        .strip_prefix("// target ")
        .map(|s| s.trim().to_string())
}

/// Parse run directive from a line.
pub fn parse_run_directive_line(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    trimmed
        .strip_prefix("// run:")
        .or_else(|| trimmed.strip_prefix("// #run:"))
        .map(|s| s.trim())
}

/// Parse a single `// run:` line into a `RunDirective`.
pub fn parse_run_directive(line: &str, line_number: usize) -> Result<RunDirective> {
    // Parse format: <expression> == <expected> or <expression> ~= <expected> [ (tolerance: <value>) ]
    let (comparison, expr, expected_with_tolerance) = if let Some(pos) = line.rfind(" == ") {
        let expr = line[..pos].trim();
        let expected = line[pos + 4..].trim();
        (ComparisonOp::Exact, expr, expected)
    } else if let Some(pos) = line.rfind(" ~= ") {
        let expr = line[..pos].trim();
        let expected = line[pos + 4..].trim();
        (ComparisonOp::Approx, expr, expected)
    } else {
        anyhow::bail!(
            "invalid run directive format at line {}: expected '==' or '~='",
            line_number
        );
    };

    // Parse tolerance if present: (tolerance: 0.001)
    let (expected, tolerance) =
        if let Some(tolerance_start) = expected_with_tolerance.find("(tolerance:") {
            let expected = expected_with_tolerance[..tolerance_start].trim();
            let tolerance_str = expected_with_tolerance[tolerance_start..]
                .strip_prefix("(tolerance:")
                .and_then(|s| s.strip_suffix(")"))
                .map(|s| s.trim());

            let tolerance = if let Some(tol_str) = tolerance_str {
                Some(tol_str.parse::<f32>().map_err(|e| {
                    anyhow::anyhow!("invalid tolerance value at line {}: {}", line_number, e)
                })?)
            } else {
                None
            };

            (expected, tolerance)
        } else {
            (expected_with_tolerance, None)
        };

    Ok(RunDirective {
        expression_str: expr.to_string(),
        comparison,
        expected_str: expected.to_string(),
        tolerance,
        line_number,
    })
}

use crate::filetest::TrapExpectation;

/// Parse trap expectation from a line.
/// Supports `// EXPECT_TRAP: <message>` or `// EXPECT_TRAP_CODE: <code>`
pub fn parse_trap_expectation(line: &str, line_number: usize) -> Result<Option<TrapExpectation>> {
    let trimmed = line.trim();

    if let Some(message) = trimmed.strip_prefix("// EXPECT_TRAP:") {
        Ok(Some(TrapExpectation {
            trap_code: None,
            trap_message: Some(message.trim().to_string()),
            line_number,
        }))
    } else if let Some(code_str) = trimmed.strip_prefix("// EXPECT_TRAP_CODE:") {
        let code = code_str
            .trim()
            .parse::<u8>()
            .map_err(|e| anyhow::anyhow!("invalid trap code at line {}: {}", line_number, e))?;
        Ok(Some(TrapExpectation {
            trap_code: Some(code),
            trap_message: None,
            line_number,
        }))
    } else {
        Ok(None)
    }
}
