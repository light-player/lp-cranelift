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
    // Parse format: <expression> == <expected> or <expression> ~= <expected>
    let (comparison, expr, expected) = if let Some(pos) = line.rfind(" == ") {
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

    Ok(RunDirective {
        expression_str: expr.to_string(),
        comparison,
        expected_str: expected.to_string(),
        line_number,
    })
}