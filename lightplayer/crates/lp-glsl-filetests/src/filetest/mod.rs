//! Test file parsing and directive extraction.

pub mod directives;
pub mod extraction;

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
    /// Test types requested in this file.
    pub test_types: Vec<TestType>,
    /// CLIF expectations extracted from comments.
    pub clif_expectations: ClifExpectations,
}

/// Test type directive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestType {
    /// `test compile` - verify CLIF IR before transformations
    Compile,
    /// `test transform.fixed32` - verify CLIF IR after fixed32 transformation
    TransformFixed32,
    /// `test run` - execute and verify results
    Run,
}

/// CLIF expectations extracted from test file comments.
#[derive(Debug, Clone, Default)]
pub struct ClifExpectations {
    /// Pre-transform CLIF (for `test compile`).
    pub pre_transform: Option<String>,
    /// Post-transform fixed32 CLIF (for `test transform.fixed32`).
    pub post_transform_fixed32: Option<String>,
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

    let lines: Vec<String> = contents.lines().map(|s| s.to_string()).collect();
    let mut test_types = Vec::new();
    let mut run_directives = Vec::new();
    let mut target = None;
    let mut is_test_run = false;

    // First pass: collect directives
    for (line_num, line) in lines.iter().enumerate() {
        if let Some(test_type) = directives::parse_test_type(line) {
            test_types.push(test_type);
            if matches!(test_type, TestType::Run) {
                is_test_run = true;
            }
            continue;
        }

        if let Some(t) = directives::parse_target_directive(line) {
            target = Some(t);
            continue;
        }

        if let Some(run_line) = directives::parse_run_directive_line(line) {
            let directive = directives::parse_run_directive(run_line, line_num + 1)?;
            run_directives.push(directive);
            continue;
        }
    }

    // Second pass: extract GLSL source and CLIF expectations
    let (glsl_source, clif_expectations) =
        extraction::extract_source_and_expectations(&lines, &test_types)?;

    Ok(TestFile {
        glsl_source,
        run_directives,
        target,
        is_test_run,
        test_types,
        clif_expectations,
    })
}

#[cfg(test)]
mod tests {
    use super::ComparisonOp;
    use super::directives;

    #[test]
    fn test_parse_run_directive() {
        let dir = directives::parse_run_directive("add_int(0, 0) == 0", 1).unwrap();
        assert_eq!(dir.expression_str, "add_int(0, 0)");
        assert_eq!(dir.comparison, ComparisonOp::Exact);
        assert_eq!(dir.expected_str, "0");

        let dir = directives::parse_run_directive("add_float(1.5, 2.5) ~= 4.0", 2).unwrap();
        assert_eq!(dir.expression_str, "add_float(1.5, 2.5)");
        assert_eq!(dir.comparison, ComparisonOp::Approx);
        assert_eq!(dir.expected_str, "4.0");
    }
}



