//! Filecheck directive parsing and matching using the filecheck crate

use filecheck::{Checker, CheckerBuilder, NO_VARIABLES};

/// Build a filechecker from expected text containing directives
pub fn build_filechecker(expected_text: &str) -> Result<Checker, String> {
    let mut builder = CheckerBuilder::new();

    // Parse each line and add directives
    for line in expected_text.lines() {
        let trimmed = line.trim();
        // Skip empty lines
        if trimmed.is_empty() {
            continue;
        }

        // Skip check: directives that start with '#' - these are section markers
        // used for organizing tests, not actual patterns to match against output.
        // Example: "check: # Prologue" is just a comment, not a pattern to verify.
        // This allows test files to have organizational comments without filecheck errors.
        if trimmed.starts_with("check:") {
            let pattern = trimmed[6..].trim();
            if pattern.starts_with('#') {
                // This is a comment/section marker - skip it
                continue;
            }
        }

        // Add directive - filecheck crate will parse it
        builder
            .directive(trimmed)
            .map_err(|e| format!("Failed to parse filecheck directive '{}': {}", trimmed, e))?;
    }

    Ok(builder.finish())
}

/// Match actual output against filecheck directives
pub fn match_filecheck(actual: &str, expected_text: &str) -> Result<(), String> {
    let checker = build_filechecker(expected_text)?;

    if checker
        .check(actual, NO_VARIABLES)
        .map_err(|e| format!("Filecheck error: {}", e))?
    {
        Ok(())
    } else {
        // Get explanation for why matching failed
        let (_, explain) = checker
            .explain(actual, NO_VARIABLES)
            .map_err(|e| format!("Failed to get filecheck explanation: {}", e))?;

        Err(format!("Filecheck failed:\n{}", explain))
    }
}

