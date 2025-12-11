//! Test that compilation fails with expected error

use anyhow::{Result, bail};
use std::path::Path;
use std::string::ToString;

#[derive(Debug, Default)]
struct ErrorExpectations {
    error_code: Option<String>,
    error_message: Option<String>,
    location: Option<String>,
    span_text: Option<String>,
    note: Option<String>,
}

pub fn run_test(path: &Path, full_source: &str, glsl_source: &str) -> Result<()> {
    // Extract expected error expectations
    let expectations = extract_error_expectations(full_source)?;

    // Compile and expect failure - use JIT directly to get detailed error
    // Try to use host ISA, but fall back to riscv32 if host isn't available
    let mut jit =
        match crate::filetest::build_isa_for_target(crate::filetest::TestTarget::Host(None)) {
            Ok(isa) => lp_glsl::JIT::new_with_isa(isa),
            Err(_) => {
                // Host ISA not available, use riscv32 instead
                let isa = crate::filetest::build_isa_for_target(
                    crate::filetest::TestTarget::Riscv32(None),
                )?;
                lp_glsl::JIT::new_with_isa(isa)
            }
        };
    match jit.compile_detailed(glsl_source) {
        Ok(_) => {
            bail!("Expected compilation to fail, but it succeeded");
        }
        Err(glsl_error) => {
            // Validate structured error information
            validate_error(&glsl_error, &expectations, path)?;
        }
    }

    Ok(())
}

fn extract_error_expectations(source: &str) -> Result<ErrorExpectations> {
    let mut expectations = ErrorExpectations::default();
    let mut found_any = false;

    for line in source.lines() {
        if let Some(comment) = line.trim().strip_prefix("//") {
            let trimmed = comment.trim();

            if let Some(code) = trimmed.strip_prefix("EXPECT_ERROR_CODE:") {
                expectations.error_code = Some(code.trim().to_string());
                found_any = true;
            } else if let Some(msg) = trimmed.strip_prefix("EXPECT_ERROR:") {
                expectations.error_message = Some(msg.trim().to_string());
                found_any = true;
            } else if let Some(msg) = trimmed.strip_prefix("EXPECT_ERROR_MSG:") {
                // Backward compatibility
                expectations.error_message = Some(msg.trim().to_string());
                found_any = true;
            } else if let Some(loc) = trimmed.strip_prefix("EXPECT_LOCATION:") {
                expectations.location = Some(loc.trim().to_string());
                found_any = true;
            } else if let Some(text) = trimmed.strip_prefix("EXPECT_SPAN_TEXT:") {
                expectations.span_text = Some(text.trim().to_string());
                found_any = true;
            } else if let Some(note) = trimmed.strip_prefix("EXPECT_NOTE:") {
                expectations.note = Some(note.trim().to_string());
                found_any = true;
            }
        }
    }

    if !found_any {
        bail!("No EXPECT_ERROR directive found (expected EXPECT_ERROR, EXPECT_ERROR_CODE, etc.)");
    }

    Ok(expectations)
}

fn validate_error(
    error: &lp_glsl::GlslError,
    expectations: &ErrorExpectations,
    path: &Path,
) -> Result<()> {
    let mut failures = Vec::new();

    // Validate error code
    if let Some(ref expected_code) = expectations.error_code {
        let actual_code = error.code.as_str();
        if actual_code != expected_code {
            failures.push(format!(
                "Error code mismatch: expected `{}`, got `{}`",
                expected_code, actual_code
            ));
        }
    }

    // Validate error message
    if let Some(ref expected_msg) = expectations.error_message {
        if !error_matches(&error.message, expected_msg) {
            failures.push(format!(
                "Error message mismatch:\n  Expected pattern: {}\n  Actual message: {}",
                expected_msg, error.message
            ));
        }
    }

    // Validate location
    if let Some(ref expected_loc) = expectations.location {
        let actual_loc = error
            .location
            .as_ref()
            .map(|l| l.to_string())
            .unwrap_or_else(|| "<unknown>".to_string());

        if !error_matches(&actual_loc, expected_loc) {
            failures.push(format!(
                "Location mismatch: expected pattern `{}`, got `{}`",
                expected_loc, actual_loc
            ));
        }
    }

    // Validate span text
    if let Some(ref expected_text) = expectations.span_text {
        let actual_text = error.span_text.as_ref().map(|s| s.as_str()).unwrap_or("");

        if !error_matches(actual_text, expected_text) {
            failures.push(format!(
                "Span text mismatch: expected pattern `{}`, got `{}`",
                expected_text, actual_text
            ));
        }
    }

    // Validate note
    if let Some(ref expected_note) = expectations.note {
        let found_note = error.notes.iter().any(|n| error_matches(n, expected_note));

        if !found_note {
            failures.push(format!(
                "Note not found: expected pattern `{}`, notes: {:?}",
                expected_note, error.notes
            ));
        }
    }

    if !failures.is_empty() {
        let error_str = error.to_string();

        // If BLESS mode is enabled, update the test file
        if crate::file_update::is_bless_enabled() {
            update_error_expectations_bless(path, error)?;
            return Ok(());
        }

        bail!(
            "Error validation failed:\n{}\n\nFull error:\n{}",
            failures.join("\n"),
            error_str
        );
    }

    Ok(())
}

fn error_matches(text: &str, pattern: &str) -> bool {
    // Simple substring match (case-insensitive)
    text.contains(pattern) || text.to_lowercase().contains(&pattern.to_lowercase())
}

fn update_error_expectations_bless(path: &Path, error: &lp_glsl::GlslError) -> Result<()> {
    use std::fs;

    let source = fs::read_to_string(path)?;
    let mut new_content = String::new();
    let mut updated_code = false;
    let mut updated_msg = false;
    let mut updated_loc = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("// EXPECT_ERROR_CODE:") {
            new_content.push_str(&format!("// EXPECT_ERROR_CODE: {}\n", error.code.as_str()));
            updated_code = true;
        } else if trimmed.starts_with("// EXPECT_ERROR:")
            || trimmed.starts_with("// EXPECT_ERROR_MSG:")
        {
            // Extract a reasonable pattern from the error message
            let pattern = extract_error_pattern_from_message(&error.message);
            new_content.push_str(&format!("// EXPECT_ERROR: {}\n", pattern));
            updated_msg = true;
        } else if trimmed.starts_with("// EXPECT_LOCATION:") {
            if let Some(ref loc) = error.location {
                new_content.push_str(&format!("// EXPECT_LOCATION: {}\n", loc));
                updated_loc = true;
            } else {
                new_content.push_str(line);
                new_content.push('\n');
            }
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }

    // Add missing directives if they weren't found
    if !updated_code {
        new_content.push_str(&format!("// EXPECT_ERROR_CODE: {}\n", error.code.as_str()));
    }
    if !updated_msg {
        let pattern = extract_error_pattern_from_message(&error.message);
        new_content.push_str(&format!("// EXPECT_ERROR: {}\n", pattern));
    }
    if !updated_loc && error.location.is_some() {
        new_content.push_str(&format!(
            "// EXPECT_LOCATION: {}\n",
            error.location.as_ref().unwrap()
        ));
    }

    fs::write(path, new_content)?;
    Ok(())
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
