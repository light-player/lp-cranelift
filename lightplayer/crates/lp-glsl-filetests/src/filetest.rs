//! Test file parsing and directive extraction.

use anyhow::{Context, Result};
use std::path::Path;

/// A parsed test file.
pub struct TestFile {
    /// The original source code (with directives filtered out for compilation).
    pub glsl_source: String,
    /// All run directives found in the file.
    pub run_directives: Vec<RunDirective>,
    /// Target specification (e.g., "riscv32.fixed32").
    pub target: Option<String>,
    /// Whether this is a "test run" file.
    pub is_test_run: bool,
}

/// A run directive parsed from a `// run:` line.
#[derive(Debug, Clone)]
pub struct RunDirective {
    /// The original expression string (e.g., "add_float(0.0, 0.0)").
    pub expression_str: String,
    /// The comparison operator.
    pub comparison: ComparisonOp,
    /// The expected value string (e.g., "0.0").
    pub expected_str: String,
    /// Line number for bless mode updates.
    pub line_number: usize,
}

/// Comparison operator for run directives.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOp {
    /// Exact equality (`==`).
    Exact,
    /// Approximate equality with tolerance (`~=`).
    Approx,
}

/// Parse a test file and extract all directives and source code.
pub fn parse_test_file(path: &Path) -> Result<TestFile> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let mut glsl_source = String::new();
    let mut run_directives = Vec::new();
    let mut target = None;
    let mut is_test_run = false;

    for (line_num, line) in contents.lines().enumerate() {
        let trimmed = line.trim();

        // Parse directives
        if trimmed.starts_with("// test run") {
            is_test_run = true;
            continue;
        }

        if let Some(t) = trimmed.strip_prefix("// target ") {
            target = Some(t.trim().to_string());
            continue;
        }

        if let Some(run_line) = trimmed.strip_prefix("// run:") {
            // Parse run directive
            let directive = parse_run_directive(run_line.trim(), line_num + 1)?;
            run_directives.push(directive);
            continue;
        }

        // Not a directive, add to GLSL source
        glsl_source.push_str(line);
        glsl_source.push('\n');
    }

    Ok(TestFile {
        glsl_source,
        run_directives,
        target,
        is_test_run,
    })
}

/// Parse a single `// run:` line into a `RunDirective`.
fn parse_run_directive(line: &str, line_number: usize) -> Result<RunDirective> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_run_directive() {
        let dir = parse_run_directive("add_int(0, 0) == 0", 1).unwrap();
        assert_eq!(dir.expression_str, "add_int(0, 0)");
        assert_eq!(dir.comparison, ComparisonOp::Exact);
        assert_eq!(dir.expected_str, "0");

        let dir = parse_run_directive("add_float(1.5, 2.5) ~= 4.0", 2).unwrap();
        assert_eq!(dir.expression_str, "add_float(1.5, 2.5)");
        assert_eq!(dir.comparison, ComparisonOp::Approx);
        assert_eq!(dir.expected_str, "4.0");
    }
}
