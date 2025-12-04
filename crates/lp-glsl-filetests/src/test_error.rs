//! Test that compilation fails with expected error

use anyhow::{bail, Result};
use std::path::Path;

pub fn run_test(path: &Path, full_source: &str, glsl_source: &str) -> Result<()> {
    // Extract expected error pattern
    let error_pattern = extract_error_pattern(full_source)?;

    // Compile and expect failure
    let mut compiler = lp_glsl::Compiler::new();
    match compiler.compile_int(glsl_source) {
        Ok(_) => {
            bail!("Expected compilation to fail, but it succeeded");
        }
        Err(e) => {
            let error_str = e.to_string();
            
            // Check that error matches expected pattern
            if !error_matches(&error_str, &error_pattern) {
                // If BLESS mode is enabled, update the test file
                if crate::file_update::is_bless_enabled() {
                    // Extract a reasonable pattern from the error
                    let new_pattern = extract_error_pattern_from_message(&error_str);
                    crate::file_update::update_error_expectation(path, &new_pattern)?;
                    return Ok(());
                }

                bail!(
                    "Error mismatch:\n\
                     Expected pattern: {}\n\
                     Actual error: {}\n\
                     \n\
                     This test assertion can be automatically updated by setting the\n\
                     CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                    error_pattern,
                    error_str
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
    error.contains(pattern) || error.to_lowercase().contains(&pattern.to_lowercase())
}

/// Extract a reasonable error pattern from the full error message
/// This tries to get the most relevant part without being too specific
fn extract_error_pattern_from_message(error: &str) -> String {
    // Try to extract just the main error message, without file paths or positions
    // For now, just use the first line or first sentence
    let first_line = error.lines().next().unwrap_or(error);
    
    // If the line is very long, try to get just the key part
    if first_line.len() > 100 {
        // Try to extract text after common prefixes
        for prefix in &["Error: ", "error: ", "Failed to ", "Cannot "] {
            if let Some(stripped) = first_line.strip_prefix(prefix) {
                return stripped.to_string();
            }
        }
    }
    
    first_line.to_string()
}
