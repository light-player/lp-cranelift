//! Test execution and result comparison.

pub mod bootstrap;
pub mod execution;
pub mod function_filter;
pub mod target;
pub mod value_ops;

use crate::file_update::format_glsl_value;

use crate::file_update::FileUpdate;
use crate::filetest::TestFile;
use anyhow::{Context, Result};
use lp_glsl::GlslOptions;
use lp_glsl::glsl_emu_riscv32_with_metadata;
use std::env;
use std::path::Path;

/// Run all tests in a test file.
pub fn run_test_file(test_file: &TestFile, path: &Path) -> Result<()> {
    run_test_file_with_line_filter(test_file, path, None, true)
}

/// Run all tests in a test file with optional line number filtering.
pub fn run_test_file_with_line_filter(
    test_file: &TestFile,
    path: &Path,
    line_filter: Option<usize>,
    show_full_output: bool,
) -> Result<()> {
    if !test_file.is_test_run {
        // Not a test run file, skip
        return Ok(());
    }

    // Read the original file lines to pass to bootstrap generation
    let file_contents = std::fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    let file_lines: Vec<String> = file_contents.lines().map(|s| s.to_string()).collect();

    // Compute relative path for rerun command
    let filetests_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("filetests");
    let relative_path = path
        .strip_prefix(&filetests_dir)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    // Use provided line filter
    let test_line_filter = line_filter;

    // Determine target and options
    let target_str = test_file.target.as_deref().unwrap_or("riscv32.fixed32");
    let (run_mode, decimal_format) = target::parse_target(target_str)?;

    let options = GlslOptions {
        run_mode,
        decimal_format,
    };

    let bless_enabled = env::var("CRANELIFT_TEST_BLESS").unwrap_or_default() == "1";
    let file_update = FileUpdate::new(path);

    // Process each run directive
    for directive in &test_file.run_directives {
        // Filter by line number if TEST_LINE is set
        if let Some(filter_line) = test_line_filter {
            if directive.line_number != filter_line {
                continue;
            }
        }
        // Generate bootstrap code with span information
        let bootstrap_result = bootstrap::generate_bootstrap(
            &file_lines,
            directive.line_number,
            &directive.expression_str,
        )?;

        // Compile and execute
        // Note: bootstrap_result.source now contains ONLY the function being tested + main()
        let mut executable = glsl_emu_riscv32_with_metadata(
            &bootstrap_result.source,
            options.clone(),
            Some(relative_path.clone()),
        )
        .map_err(|e| {
            format_compilation_error(
                &e,
                &bootstrap_result,
                directive.line_number,
                &directive.expression_str,
                &relative_path,
                show_full_output,
            )
        })?;

        // Check if this test expects a trap
        // Trap expectations can be on the same line or the immediately following line
        let trap_expectation = test_file.trap_expectations.iter().find(|exp| {
            exp.line_number == directive.line_number || exp.line_number == directive.line_number + 1
        });

        // Execute main() and get result
        // Note: execute_main already includes emulator state in the error, so we don't add it again
        let execution_result = execution::execute_main(&mut *executable);

        match (execution_result, trap_expectation) {
            (Ok(actual_value), Some(exp)) => {
                // Expected a trap but got a value
                let bootstrap_code_display = if show_full_output {
                    format!(
                        "\n\n=== Bootstrapped GLSL Test ===\n{}",
                        format_code_block(&bootstrap_result.source)
                    )
                } else {
                    String::new()
                };
                anyhow::bail!(
                    "run test failed at line {}: expected trap but execution succeeded\n\
                     \n\
                     Expected: trap{}\n\
                     Actual: value {}\n\
                     {}\
                     \n\
                     To rerun just this test:\n\
                     scripts/glsl-filetests.sh {}:{}",
                    directive.line_number,
                    if let Some(code) = exp.trap_code {
                        format!(" (code {})", code)
                    } else if let Some(ref msg) = exp.trap_message {
                        format!(" (message containing '{}')", msg)
                    } else {
                        String::new()
                    },
                    format_glsl_value(&actual_value),
                    bootstrap_code_display,
                    relative_path,
                    directive.line_number
                );
            }
            (Err(e), None) => {
                // Got an error but didn't expect one - check if it's a trap
                let error_str = format!("{:#}", e);
                let is_trap = error_str.contains("Trap:")
                    || error_str.contains("trap")
                    || error_str.contains("execution trapped");

                if is_trap {
                    // Unexpected trap
                    let bootstrap_code_display = if show_full_output {
                        format!(
                            "\n\n=== Bootstrapped GLSL Test ===\n{}",
                            format_code_block(&bootstrap_result.source)
                        )
                    } else {
                        String::new()
                    };
                    anyhow::bail!(
                        "run test failed at line {}: unexpected trap\n\
                         \n\
                         Expected: value\n\
                         Actual: trap\n\
                         {}\
                         \n\
                         Error details:\n\
                         {}\n\
                         \n\
                         To rerun just this test:\n\
                         scripts/glsl-filetests.sh {}:{}",
                        directive.line_number,
                        bootstrap_code_display,
                        error_str,
                        relative_path,
                        directive.line_number
                    );
                } else {
                    // Other error - pass through
                    return Err(e);
                }
            }
            (Err(e), Some(exp)) => {
                // Expected a trap and got one - verify it matches
                let error_str = format!("{:#}", e);

                // Check trap code if specified
                if let Some(expected_code) = exp.trap_code {
                    if !error_str.contains(&format!("user{}", expected_code)) {
                        let bootstrap_code_display = if show_full_output {
                            format!(
                                "\n\n=== Bootstrapped GLSL Test ===\n{}",
                                format_code_block(&bootstrap_result.source)
                            )
                        } else {
                            String::new()
                        };
                        anyhow::bail!(
                            "run test failed at line {}: trap code mismatch\n\
                             \n\
                             Expected: trap code {}\n\
                             Actual trap: {}\n\
                             {}\
                             \n\
                             To rerun just this test:\n\
                             scripts/glsl-filetests.sh {}:{}",
                            directive.line_number,
                            expected_code,
                            error_str,
                            bootstrap_code_display,
                            relative_path,
                            directive.line_number
                        );
                    }
                }

                // Check trap message if specified
                if let Some(ref expected_msg) = exp.trap_message {
                    if !error_str.contains(expected_msg) {
                        let bootstrap_code_display = if show_full_output {
                            format!(
                                "\n\n=== Bootstrapped GLSL Test ===\n{}",
                                format_code_block(&bootstrap_result.source)
                            )
                        } else {
                            String::new()
                        };
                        anyhow::bail!(
                            "run test failed at line {}: trap message mismatch\n\
                             \n\
                             Expected: trap message containing '{}'\n\
                             Actual trap: {}\n\
                             {}\
                             \n\
                             To rerun just this test:\n\
                             scripts/glsl-filetests.sh {}:{}",
                            directive.line_number,
                            expected_msg,
                            error_str,
                            bootstrap_code_display,
                            relative_path,
                            directive.line_number
                        );
                    }
                }

                // Trap matches expectation - test passes
                continue;
            }
            (Ok(actual_value), None) => {
                // Normal case: expected value, got value - continue with comparison
                // Parse expected value
                let expected_value = value_ops::parse_glsl_value(&directive.expected_str)?;

                // Compare results
                match value_ops::compare_results(
                    &actual_value,
                    &expected_value,
                    directive.comparison,
                ) {
                    Ok(()) => {
                        // Test passed
                    }
                    Err(err_msg) => {
                        if bless_enabled {
                            // Update expectation in-place
                            file_update.update_run_expectation(
                                directive.line_number,
                                &actual_value,
                                directive.comparison,
                            )?;
                        } else {
                            // Format bootstrap code for display (only when showing full output)
                            let bootstrap_code_display = if show_full_output {
                                format!(
                                    "\n\n=== Bootstrapped GLSL Test ===\n{}",
                                    format_code_block(&bootstrap_result.source)
                                )
                            } else {
                                String::new()
                            };

                            // Format debug information (CLIF IR, VCode, disassembly, emulator state)
                            // when showing full output
                            let debug_info_display = if show_full_output {
                                format_debug_info(&*executable)
                            } else {
                                String::new()
                            };

                            // Generate rerun command using the script
                            let rerun_cmd = format!(
                                "scripts/glsl-filetests.sh {}:{}",
                                relative_path, directive.line_number
                            );

                            // Format error message
                            // Note: For comparison errors, execution succeeded so emulator state isn't needed
                            let error_msg = format!(
                                "run test failed at line {}: {}{}{}\n\
                                 \n\
                                 This test assertion can be automatically updated by setting the\n\
                                 CRANELIFT_TEST_BLESS=1 environment variable when running this test.\n\
                                 \n\
                                 To rerun just this test:\n\
                                 {}",
                                directive.line_number,
                                err_msg,
                                bootstrap_code_display,
                                debug_info_display,
                                rerun_cmd
                            );
                            anyhow::bail!("{}", error_msg);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Format a compilation error with bootstrap code context.
/// This is a thin wrapper that only adds test-specific context.
/// All error formatting is delegated to GlslError::Display.
fn format_compilation_error(
    error: &lp_glsl::error::GlslError,
    bootstrap: &bootstrap::BootstrapResult,
    directive_line: usize,
    expression: &str,
    relative_path: &str,
    show_full_output: bool,
) -> anyhow::Error {
    // Generate rerun command using the script
    let rerun_cmd = format!(
        "scripts/glsl-filetests.sh {}:{}",
        relative_path, directive_line
    );

    // Get the fully formatted error from GlslError::Display (single source of truth)
    let formatted_error = error.to_string();

    // Format bootstrap code only when showing full output
    let bootstrap_section = if show_full_output {
        format!(
            "\n\n=== Bootstrapped GLSL Test ===\n{}\n",
            format_code_block(&bootstrap.source)
        )
    } else {
        String::new()
    };

    // Build the error message with test-specific context
    // Add a blank line between bootstrap code and error message
    let mut msg = format!(
        "Compilation failed for test case at line {}:\n\
         \n\
         Test case: {}{}\
         \n\
         {}",
        directive_line, expression, bootstrap_section, formatted_error
    );

    // Add rerun command (formatted_error already ends with \n, so \n here creates one blank line)
    msg.push_str(&format!("\nTo rerun just this test:\n{}", rerun_cmd));

    anyhow::anyhow!("{}", msg)
}

/// Format all debug information from an executable (CLIF IR, VCode, disassembly, emulator state)
fn format_debug_info(executable: &dyn lp_glsl::GlslExecutable) -> String {
    let mut parts = Vec::new();

    // Get CLIF IR (before and after transformation)
    let (original_clif, transformed_clif) = executable.format_clif_ir();

    // Only show before/after if they're different
    match (&original_clif, &transformed_clif) {
        (Some(original), Some(transformed)) if original != transformed => {
            // They're different, show both
            parts.push(format!(
                "=== CLIF IR (BEFORE transformation) ===\n{}",
                original
            ));
            parts.push(format!(
                "=== CLIF IR (AFTER transformation) ===\n{}",
                transformed
            ));
        }
        (Some(original), Some(_)) => {
            // They're the same, just show one
            parts.push(format!("=== CLIF IR ===\n{}", original));
        }
        (Some(original), None) => {
            // Only original available
            parts.push(format!("=== CLIF IR ===\n{}", original));
        }
        (None, Some(transformed)) => {
            // Only transformed available
            parts.push(format!("=== CLIF IR ===\n{}", transformed));
        }
        (None, None) => {
            // No CLIF IR available
        }
    }

    // Get VCode
    if let Some(ref vcode) = executable.format_vcode() {
        parts.push(format!("=== VCode ===\n{}", vcode));
    }

    // Get disassembly
    if let Some(ref disassembly) = executable.format_disassembly() {
        parts.push(format!("=== Disassembled ===\n{}", disassembly));
    }

    // Get emulator state
    if let Some(ref emulator_state) = executable.format_emulator_state() {
        parts.push(emulator_state.clone());
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("\n\n{}", parts.join("\n\n"))
    }
}

/// Format source code as a code block with line numbers for better readability
fn format_code_block(source: &str) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let max_line_num_width = (lines.len() + 1).to_string().len();

    lines
        .iter()
        .enumerate()
        .map(|(i, line)| format!("{:width$} | {}", i + 1, line, width = max_line_num_width))
        .collect::<Vec<_>>()
        .join("\n")
}
