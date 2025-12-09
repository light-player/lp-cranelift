//! Test CLIF IR generation
//! Pattern: cranelift/filetests/src/subtest.rs

use anyhow::{bail, Result};
use std::path::Path;
use regex::Regex;

pub fn run_test(
    path: &Path,
    full_source: &str,
    glsl_source: &str,
    fixed_point_format: Option<lp_glsl::FixedPointFormat>,
) -> Result<()> {
    // Parse target directives from the test file
    let targets = crate::filetest::parse_target_directives(full_source)?;
    
    // Use first target if specified, otherwise try host and fall back to riscv32
    let target = targets.first().copied().unwrap_or_else(|| {
        // Default to host, but will fall back to riscv32 if host fails
        crate::filetest::TestTarget::Host(None)
    });
    
    // Build ISA for the target, with fallback to riscv32 if host isn't available
    let isa = match crate::filetest::build_isa_for_target(target) {
        Ok(isa) => isa,
        Err(_) if target.is_host() => {
            // Host ISA not available, fall back to riscv32
            crate::filetest::build_isa_for_target(crate::filetest::TestTarget::Riscv32(None))?
        }
        Err(e) => return Err(e),
    };
    
    // Create JIT with target ISA
    let mut jit = lp_glsl::JIT::new_with_isa(isa);
    jit.fixed_point_format = fixed_point_format;
    let clif = jit
        .compile_to_clif(glsl_source)
        .map_err(|e| anyhow::anyhow!("Compilation failed: {}", e))?;

    // Extract expected output from comments
    let expected = extract_expected_output(full_source);
    let actual = clif.trim();

    // Normalize architecture-specific calling convention names before comparison
    // CLIF IR is architecture-independent, so we normalize to make tests portable
    let expected_normalized = normalize_architecture_names(expected.trim());
    let actual_normalized = normalize_architecture_names(actual);

    // Compare expected vs actual
    if expected_normalized == actual_normalized {
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
         Expected (normalized):\n\
         {}\n\
         \n\
         Actual (normalized):\n\
         {}\n\
         \n\
         Expected (original):\n\
         {}\n\
         \n\
         Actual (original):\n\
         {}\n\
         \n\
         This test assertion can be automatically updated by setting the\n\
         CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
        expected_normalized,
        actual_normalized,
        expected.trim(),
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

/// Normalize architecture-specific calling convention names in CLIF IR output.
/// 
/// CLIF IR is architecture-independent, but the printed representation includes
/// architecture-specific calling convention names (e.g., `apple_aarch64`, `system_v`).
/// This function replaces all calling convention names with a placeholder `ARCH`
/// to make tests architecture-independent.
fn normalize_architecture_names(clif_output: &str) -> String {
    // Calling conventions appear after function signatures, e.g.:
    // "function u0:0() -> i32 apple_aarch64 {"
    // "function %foo(i32) system_v {"
    //
    // The format is: function <name>(<params>) [-> <returns>] <call_conv> {
    //
    // Known calling conventions (from cranelift_codegen::isa::CallConv):
    // - fast, cold, tail (generic)
    // - system_v (System V, used by many architectures)
    // - windows_fastcall (Windows x64/ARM)
    // - apple_aarch64 (Apple aarch64)
    // - probestack, winch, patchable (specialized)
    //
    // We match a calling convention that appears:
    // - After a closing parenthesis `)` or after `-> <type>`
    // - Before an opening brace `{`
    // - With whitespace around it
    //
    // Pattern explanation:
    // - `\s+` matches whitespace before the calling convention
    // - `(fast|cold|tail|system_v|windows_fastcall|apple_aarch64|probestack|winch|patchable)` matches known calling conventions
    // - `\s*\{` matches optional whitespace and the opening brace
    let re = Regex::new(r"\s+(fast|cold|tail|system_v|windows_fastcall|apple_aarch64|probestack|winch|patchable)\s*\{")
        .expect("Failed to compile regex pattern");
    
    re.replace_all(clif_output, " ARCH {").to_string()
}
