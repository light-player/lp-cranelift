//! Test that compilation fails with expected error

use anyhow::{bail, Result};

pub fn run_test(full_source: &str, glsl_source: &str) -> Result<()> {
    // Extract expected error pattern
    let error_pattern = extract_error_pattern(full_source)?;

    // Compile and expect failure
    let mut compiler = lp_glsl::Compiler::new();
    match compiler.compile_int(glsl_source) {
        Ok(_) => {
            bail!("Expected compilation to fail, but it succeeded");
        }
        Err(e) => {
            // Check that error matches expected pattern
            if !error_matches(&e, &error_pattern) {
                bail!(
                    "Error mismatch:\nExpected pattern: {}\nActual error: {}",
                    error_pattern,
                    e
                );
            }
        }
    }

    Ok(())
}

fn extract_error_pattern(source: &str) -> Result<String> {
    for line in source.lines() {
        if let Some(comment) = line.trim().strip_prefix("//") {
            if let Some(pattern) = comment.trim().strip_prefix("EXPECT_ERROR:") {
                return Ok(pattern.trim().to_string());
            }
        }
    }
    bail!("No EXPECT_ERROR directive found")
}

fn error_matches(error: &str, pattern: &str) -> bool {
    // Simple substring match (could use regex in future)
    error.contains(pattern) ||
    error.to_lowercase().contains(&pattern.to_lowercase())
}

