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
        let trimmed = line.trim();

        // Parse test type directives
        if trimmed == "// test compile" {
            test_types.push(TestType::Compile);
            continue;
        }

        if trimmed == "// test transform.fixed32" {
            test_types.push(TestType::TransformFixed32);
            continue;
        }

        if trimmed.starts_with("// test run") {
            test_types.push(TestType::Run);
            is_test_run = true;
            continue;
        }

        if let Some(t) = trimmed.strip_prefix("// target ") {
            target = Some(t.trim().to_string());
            continue;
        }

        if let Some(run_line) = trimmed.strip_prefix("// run:") {
            let directive = parse_run_directive(run_line.trim(), line_num + 1)?;
            run_directives.push(directive);
            continue;
        }
    }

    // Second pass: extract GLSL source and CLIF expectations
    let (glsl_source, clif_expectations) = extract_source_and_expectations(&lines, &test_types)?;

    Ok(TestFile {
        glsl_source,
        run_directives,
        target,
        is_test_run,
        test_types,
        clif_expectations,
    })
}

/// Extract GLSL source code and CLIF expectations from file lines.
fn extract_source_and_expectations(
    lines: &[String],
    test_types: &[TestType],
) -> Result<(String, ClifExpectations)> {
    let mut glsl_source = String::new();
    let mut clif_expectations = ClifExpectations::default();

    // Find the end of GLSL code (last non-comment, non-directive line)
    let mut glsl_end = 0;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("//")
            && !trimmed.starts_with("// test")
            && !trimmed.starts_with("// target")
            && !trimmed.starts_with("// run:")
        {
            glsl_end = i + 1;
        }
    }

    // Extract GLSL source (everything before expectations)
    for line in lines.iter().take(glsl_end) {
        let trimmed = line.trim();
        // Skip directives
        if trimmed.starts_with("// test") || trimmed.starts_with("// target") {
            continue;
        }
        glsl_source.push_str(line);
        glsl_source.push('\n');
    }

    // Find the start of run expectations (first `// #run:` or `// run:` after GLSL)
    let mut run_start = lines.len();
    for (i, line) in lines.iter().enumerate().skip(glsl_end) {
        let trimmed = line.trim();
        if trimmed.starts_with("// #run:") || trimmed.starts_with("// run:") {
            run_start = i;
            break;
        }
    }

    // Extract CLIF expectations from the section between GLSL and run expectations
    if run_start > glsl_end {
        let clif_section: Vec<&String> = lines[glsl_end..run_start].iter().collect();
        clif_expectations = parse_clif_expectations(&clif_section, test_types)?;
    }

    Ok((glsl_source, clif_expectations))
}

/// Parse CLIF expectations from comment lines.
fn parse_clif_expectations(lines: &[&String], test_types: &[TestType]) -> Result<ClifExpectations> {
    let mut expectations = ClifExpectations::default();

    // Look for section markers
    let mut compile_section_start = None;
    let mut transform_section_start = None;
    let mut compile_section_end = None;
    let mut transform_section_end = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("// #compile:") {
            compile_section_start = Some(i + 1);
        } else if trimmed.starts_with("// #transform:") {
            if compile_section_start.is_some() && compile_section_end.is_none() {
                compile_section_end = Some(i);
            }
            transform_section_start = Some(i + 1);
        } else if trimmed == "//"
            && compile_section_start.is_some()
            && compile_section_end.is_none()
        {
            // Blank comment line after compile section
            compile_section_end = Some(i);
        } else if trimmed == "//"
            && transform_section_start.is_some()
            && transform_section_end.is_none()
        {
            // Blank comment line after transform section
            transform_section_end = Some(i);
        }
    }

    // If no markers found, try to split by blank comment lines
    if compile_section_start.is_none() && transform_section_start.is_none() {
        let has_compile = test_types.contains(&TestType::Compile);
        let has_transform = test_types.contains(&TestType::TransformFixed32);

        if has_compile && has_transform {
            // Split by blank comment line
            let mut blank_line_idx = None;
            for (i, line) in lines.iter().enumerate() {
                if line.trim() == "//" {
                    blank_line_idx = Some(i);
                    break;
                }
            }
            if let Some(blank) = blank_line_idx {
                expectations.pre_transform = Some(extract_clif_from_lines(&lines[..blank]));
                expectations.post_transform_fixed32 =
                    Some(extract_clif_from_lines(&lines[blank + 1..]));
            } else if has_compile {
                expectations.pre_transform = Some(extract_clif_from_lines(lines));
            } else if has_transform {
                expectations.post_transform_fixed32 = Some(extract_clif_from_lines(lines));
            }
        } else if has_compile {
            expectations.pre_transform = Some(extract_clif_from_lines(lines));
        } else if has_transform {
            expectations.post_transform_fixed32 = Some(extract_clif_from_lines(lines));
        }
    } else {
        // Extract sections based on markers
        if let Some(start) = compile_section_start {
            let end = compile_section_end.unwrap_or(lines.len());
            if test_types.contains(&TestType::Compile) {
                expectations.pre_transform = Some(extract_clif_from_lines(&lines[start..end]));
            }
        }

        if let Some(start) = transform_section_start {
            let end = transform_section_end.unwrap_or(lines.len());
            if test_types.contains(&TestType::TransformFixed32) {
                expectations.post_transform_fixed32 =
                    Some(extract_clif_from_lines(&lines[start..end]));
            }
        }
    }

    Ok(expectations)
}

/// Extract CLIF text from comment lines (removing `//` prefix).
fn extract_clif_from_lines(lines: &[&String]) -> String {
    let mut result = String::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed == "//" {
            result.push('\n');
        } else if let Some(clif_line) = trimmed.strip_prefix("// ") {
            result.push_str(clif_line);
            result.push('\n');
        } else if trimmed.starts_with("//") {
            // Handle lines that are just "//" or have content after "//"
            let clif_line = trimmed.strip_prefix("//").unwrap_or(trimmed).trim();
            if !clif_line.is_empty() {
                result.push_str(clif_line);
                result.push('\n');
            } else {
                result.push('\n');
            }
        }
    }
    result.trim_end().to_string()
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
