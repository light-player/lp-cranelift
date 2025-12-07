//! Test CLIF IR generation
//! Pattern: cranelift/filetests/src/subtest.rs

use anyhow::{bail, Result};
use std::path::Path;

pub fn run_test(
    path: &Path,
    full_source: &str,
    glsl_source: &str,
    fixed_point_format: Option<lp_glsl::FixedPointFormat>,
) -> Result<()> {
    // Compile GLSL to CLIF
    let mut jit = lp_glsl::JIT::new();
    jit.fixed_point_format = fixed_point_format;
    let clif = jit
        .compile_to_clif(glsl_source)
        .map_err(|e| anyhow::anyhow!("Compilation failed: {}", e))?;

    // Extract expected output from comments
    let expected = extract_expected_output(full_source);
    let actual = clif.trim();

    // Compare expected vs actual
    if expected.trim() == actual {
        return Ok(());
    }

    // If BLESS mode is enabled, update the test file
    if crate::file_update::is_bless_enabled() {
        crate::file_update::update_compile_expectations(path, actual)?;
        return Ok(());
    }

    // Otherwise, report the mismatch
    bail!(
        "CLIF output does not match expectation.\n\
         \n\
         Expected:\n\
         {}\n\
         \n\
         Actual:\n\
         {}\n\
         \n\
         This test assertion can be automatically updated by setting the\n\
         CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
        expected,
        actual
    )
}

/// Extract expected CLIF output from trailing `//` comments
fn extract_expected_output(source: &str) -> String {
    let mut expected_lines = Vec::new();
    let mut in_expectations = false;

    for line in source.lines() {
        let trimmed = line.trim();
        
        // Check if this is a comment line that's part of expectations
        if let Some(comment_content) = trimmed.strip_prefix("//") {
            // Only strip one space after // if present, preserve the rest
            let content = if let Some(c) = comment_content.strip_prefix(' ') {
                c
            } else {
                comment_content
            };
            
            // Skip test directives and other special comments
            if content.trim_start().starts_with("test ")
                || content.trim_start().starts_with("CHECK")
                || content.trim_start().starts_with("run:")
                || content.trim_start().starts_with("EXPECT_ERROR:")
                || content.trim_start().starts_with("Validate")
            {
                continue;
            }
            
            // Only start collecting expectations when we see CLIF-like patterns
            // (function declarations or block labels)
            if !in_expectations {
                if content.trim_start().starts_with("function")
                    || content.trim_start().starts_with("block") {
                    in_expectations = true;
                } else {
                    // Skip explanatory comments that don't look like CLIF
                    continue;
                }
            }
            
            // This is an expectation comment
            expected_lines.push(content.to_string());
        } else if in_expectations && !trimmed.is_empty() {
            // We hit a non-comment line after starting expectations, stop
            break;
        }
    }

    expected_lines.join("\n")
}
