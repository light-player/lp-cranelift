//! Utilities for updating test file expectations (BLESS mode)

use anyhow::{Context, Result};
use std::path::Path;

/// Check if BLESS mode is enabled via environment variable
pub fn is_bless_enabled() -> bool {
    std::env::var("CRANELIFT_TEST_BLESS").unwrap_or_default() == "1"
}

/// Update compile test expectations with new CLIF output
/// Replaces CLIF expectation comments while preserving run/error directives
pub fn update_compile_expectations(path: &Path, clif_output: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    let lines: Vec<&str> = source.lines().collect();
    let mut new_content = String::new();
    
    // Collect run and error directives to preserve
    let mut preserved_directives = Vec::new();
    for line in &lines {
        let trimmed = line.trim();
        if trimmed.starts_with("// run:") || trimmed.starts_with("// EXPECT_ERROR:") {
            preserved_directives.push(line.to_string());
        }
    }
    
    // Find the closing brace of the main function (or last non-comment code line)
    let mut last_code_line = 0;
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        // This is a code line if it's not empty and doesn't start with //
        if !trimmed.is_empty() && !trimmed.starts_with("//") {
            last_code_line = idx;
        }
    }
    
    // Write original GLSL source and test directives
    for idx in 0..=last_code_line {
        new_content.push_str(lines[idx]);
        new_content.push('\n');
    }
    
    // Add blank line before expectations
    new_content.push('\n');
    
    // Add new CLIF output as comments
    for line in clif_output.lines() {
        if line.is_empty() {
            new_content.push_str("//\n");
        } else {
            new_content.push_str("// ");
            new_content.push_str(line);
            new_content.push('\n');
        }
    }
    
    // Add preserved directives if any
    for directive in preserved_directives {
        new_content.push_str(&directive);
        new_content.push('\n');
    }
    
    // Ensure file ends with newline
    if !new_content.ends_with('\n') {
        new_content.push('\n');
    }
    
    std::fs::write(path, new_content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    
    Ok(())
}

/// Update a run directive with new value
pub fn update_run_directive(path: &Path, new_directive: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    let mut new_content = String::new();
    let mut updated = false;
    
    for line in source.lines() {
        if line.trim().starts_with("// run:") && !updated {
            // Replace with new directive
            new_content.push_str("// run: ");
            new_content.push_str(new_directive);
            new_content.push('\n');
            updated = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }
    
    std::fs::write(path, new_content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    
    Ok(())
}

/// Update error expectation with new pattern
pub fn update_error_expectation(path: &Path, new_pattern: &str) -> Result<()> {
    let source = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;
    
    let mut new_content = String::new();
    let mut updated = false;
    
    for line in source.lines() {
        if line.trim().starts_with("// EXPECT_ERROR:") && !updated {
            // Replace with new pattern
            new_content.push_str("// EXPECT_ERROR: ");
            new_content.push_str(new_pattern);
            new_content.push('\n');
            updated = true;
        } else {
            new_content.push_str(line);
            new_content.push('\n');
        }
    }
    
    // If no EXPECT_ERROR line found, add it at the end
    if !updated {
        new_content.push_str("\n// EXPECT_ERROR: ");
        new_content.push_str(new_pattern);
        new_content.push('\n');
    }
    
    std::fs::write(path, new_content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    
    Ok(())
}
