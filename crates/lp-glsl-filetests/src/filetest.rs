//! Run GLSL filetests
//! Pattern: cranelift/filetests/src/runone.rs

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn run_filetest(path: &Path) -> Result<()> {
    let source = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    // Parse test directives
    let test_compile = source.contains("test compile");
    let test_run = source.contains("test run");
    let test_error = source.contains("test error");
    let test_fixed32 = source.contains("test fixed32");
    let test_fixed64 = source.contains("test fixed64");
    
    // Determine fixed-point format
    let fixed_point_format = if test_fixed32 {
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
        crate::test_compile::run_test(path, &source, &glsl_source, fixed_point_format)?;
    }
    
    if test_run {
        crate::test_run::run_test(path, &source, &glsl_source, fixed_point_format)?;
    }
    
    if !test_compile && !test_run && !test_error {
        anyhow::bail!("No test directives found (expected 'test compile', 'test run', or 'test error')");
    }
    
    Ok(())
}

fn extract_glsl_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            // Keep all lines that don't start with // test, // CHECK, // run:, or // EXPECT_ERROR:
            // This preserves regular GLSL comments
            if let Some(comment_content) = trimmed.strip_prefix("//") {
                let content = comment_content.trim();
                !content.starts_with("test ")
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

