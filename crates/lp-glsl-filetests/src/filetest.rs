//! Run GLSL filetests
//! Pattern: cranelift/filetests/src/runone.rs

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestTarget {
    Host(Option<lp_glsl::FixedPointFormat>),
    Riscv32(Option<lp_glsl::FixedPointFormat>),
}

impl TestTarget {
    pub fn fixed_point_format(&self) -> Option<lp_glsl::FixedPointFormat> {
        match self {
            TestTarget::Host(fmt) | TestTarget::Riscv32(fmt) => *fmt,
        }
    }

    pub fn is_host(&self) -> bool {
        matches!(self, TestTarget::Host(_))
    }

    pub fn is_riscv32(&self) -> bool {
        matches!(self, TestTarget::Riscv32(_))
    }
}

pub fn run_filetest(path: &Path) -> Result<()> {
    let source =
        fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;

    // Parse test directives
    let test_compile = source.contains("test compile");
    let test_run = source.contains("test run");
    let test_error = source.contains("test error");
    let test_fixed32 = source.contains("test fixed32");
    let test_fixed64 = source.contains("test fixed64");

    // Parse target directives
    let targets = parse_target_directives(&source)?;

    // If no targets specified, default to host
    let targets = if targets.is_empty() {
        vec![TestTarget::Host(None)]
    } else {
        targets
    };

    // Legacy fixed-point format support (for backward compatibility)
    // If target specifies format, use that; otherwise check legacy directives
    let legacy_fixed_point_format = if test_fixed32 {
        Some(lp_glsl::FixedPointFormat::Fixed16x16)
    } else if test_fixed64 {
        Some(lp_glsl::FixedPointFormat::Fixed32x32)
    } else {
        None
    };

    // Extract just the GLSL code (lines that don't start with ; or test directives)
    let glsl_source = extract_glsl_source(&source);

    if test_error {
        crate::test_error::run_test(path, &source, &glsl_source)?;
    }

    if test_compile {
        // For compile tests, use first target or legacy format
        let fixed_point_format = targets
            .first()
            .and_then(|t| t.fixed_point_format())
            .or(legacy_fixed_point_format);
        crate::test_compile::run_test(path, &source, &glsl_source, fixed_point_format)?;
    }

    if test_run {
        crate::test_run::run_test(path, &source, &glsl_source, &targets)?;
    }

    if !test_compile && !test_run && !test_error {
        anyhow::bail!(
            "No test directives found (expected 'test compile', 'test run', or 'test error')"
        );
    }

    Ok(())
}

fn parse_target_directives(source: &str) -> Result<Vec<TestTarget>> {
    let mut targets = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(comment_content) = trimmed.strip_prefix("//") {
            let content = comment_content.trim();
            if let Some(target_spec) = content.strip_prefix("target") {
                let target_spec = target_spec.trim();

                // Parse format: "host", "host.fixed32", "host.fixed64", "riscv32", "riscv32.fixed32", "riscv32.fixed64"
                if let Some((arch, format_str)) = target_spec.split_once('.') {
                    let fixed_point_format = match format_str {
                        "fixed32" => Some(lp_glsl::FixedPointFormat::Fixed16x16),
                        "fixed64" => Some(lp_glsl::FixedPointFormat::Fixed32x32),
                        _ => anyhow::bail!("Unknown fixed-point format: {}", format_str),
                    };

                    match arch.trim() {
                        "host" => targets.push(TestTarget::Host(fixed_point_format)),
                        "riscv32" => targets.push(TestTarget::Riscv32(fixed_point_format)),
                        _ => anyhow::bail!("Unknown target architecture: {}", arch),
                    }
                } else {
                    // No format specified
                    match target_spec.trim() {
                        "host" => targets.push(TestTarget::Host(None)),
                        "riscv32" => targets.push(TestTarget::Riscv32(None)),
                        _ => anyhow::bail!("Unknown target architecture: {}", target_spec),
                    }
                }
            }
        }
    }

    Ok(targets)
}

fn extract_glsl_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Keep all lines that don't start with // test, // target, // CHECK, // run:, or // EXPECT_ERROR:
            // This preserves regular GLSL comments
            if let Some(comment_content) = trimmed.strip_prefix("//") {
                let content = comment_content.trim();
                !content.starts_with("test ")
                    && !content.starts_with("target")
                    && !content.starts_with("CHECK")
                    && !content.starts_with("run:")
                    && !content.starts_with("EXPECT_ERROR:")
            } else {
                true
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
