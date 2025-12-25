//! Test execution and result comparison.

pub mod bootstrap;
pub mod execution;
pub mod target;
pub mod value_ops;

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
        let mut executable =
            glsl_emu_riscv32_with_metadata(&bootstrap_result.source, options.clone(), Some(relative_path.clone())).map_err(|e| {
                format_compilation_error(
                    &e,
                    &bootstrap_result,
                    directive.line_number,
                    &directive.expression_str,
                    &relative_path,
                    show_full_output,
                )
            })?;

        // Execute main() and get result
        let actual_value = execution::execute_main(&mut *executable).map_err(|e| {
            // Check if this is a trap error
            let error_str = format!("{:#}", e);
            let is_trap = error_str.contains("Trap:") || error_str.contains("trap") || error_str.contains("execution trapped");
            
            // Get emulator state if available (only when showing full output)
            let emulator_state = if show_full_output {
                executable.format_emulator_state()
            } else {
                None
            };
            
            // Get CLIF IR (before and after transformation) if available (only when showing full output)
            let clif_ir_section = if show_full_output {
                let (original_ir, transformed_ir) = executable.format_clif_ir();
                let mut section = String::new();
                if let Some(ref orig) = original_ir {
                    section.push_str("\n\n=== CLIF IR (BEFORE transformation) ===\n");
                    section.push_str(orig);
                }
                if let Some(ref trans) = transformed_ir {
                    section.push_str("\n\n=== CLIF IR (AFTER transformation) ===\n");
                    section.push_str(trans);
                }
                section
            } else {
                String::new()
            };
            
            // Generate rerun command using the script
            let rerun_cmd = format!(
                "scripts/glsl-filetests.sh {}:{}",
                relative_path, directive.line_number
            );
            
            // Format bootstrap code for display (only when showing full output)
            let bootstrap_code_display = if show_full_output {
                format!("\n\nGenerated test code:\n{}", format_code_block(&bootstrap_result.source))
            } else {
                String::new()
            };
            
            if is_trap {
                // Format trap error with clear message and bootstrap code context
                let trap_msg = if let Some(state) = emulator_state {
                    format!(
                        "run test failed at line {}: execution trapped\n\
                         \n\
                         The test expected a value but execution trapped instead.\n\
                         This indicates the code under test encountered an error condition\n\
                         (e.g., division by zero, overflow, etc.).{}{}\n\
                         \n\
                         Error details:\n\
                         {}{}\n\
                         \n\
                         To rerun just this test:\n\
                         {}",
                        directive.line_number, bootstrap_code_display, clif_ir_section, error_str, state, rerun_cmd
                    )
                } else {
                    format!(
                        "run test failed at line {}: execution trapped\n\
                         \n\
                         The test expected a value but execution trapped instead.\n\
                         This indicates the code under test encountered an error condition\n\
                         (e.g., division by zero, overflow, etc.).{}{}\n\
                         \n\
                         Error details:\n\
                         {}\n\
                         \n\
                         To rerun just this test:\n\
                         {}",
                        directive.line_number, bootstrap_code_display, clif_ir_section, error_str, rerun_cmd
                    )
                };
                anyhow::anyhow!("{}", trap_msg)
            } else {
                // Regular execution error
                let error_msg = if let Some(state) = emulator_state {
                    format!("{}{}\n\nTo rerun just this test:\n{}", error_str, state, rerun_cmd)
                } else {
                    format!("{}\n\nTo rerun just this test:\n{}", error_str, rerun_cmd)
                };
                anyhow::anyhow!("{}", error_msg)
            }
        })?;

        // Parse expected value
        let expected_value = value_ops::parse_glsl_value(&directive.expected_str)?;

        // Compare results
        match value_ops::compare_results(&actual_value, &expected_value, directive.comparison) {
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
                    // Get emulator state if available (only when showing full output)
                    let emulator_state = if show_full_output {
                        executable.format_emulator_state()
                    } else {
                        None
                    };

                    // Get CLIF IR (before and after transformation) if available (only when showing full output)
                    let clif_ir_section = if show_full_output {
                        let (original_ir, transformed_ir) = executable.format_clif_ir();
                        let mut section = String::new();
                        if let Some(ref orig) = original_ir {
                            section.push_str("\n\n=== CLIF IR (BEFORE transformation) ===\n");
                            section.push_str(orig);
                        }
                        if let Some(ref trans) = transformed_ir {
                            section.push_str("\n\n=== CLIF IR (AFTER transformation) ===\n");
                            section.push_str(trans);
                        }
                        section
                    } else {
                        String::new()
                    };

                    // Format bootstrap code for display (only when showing full output)
                    let bootstrap_code_display = if show_full_output {
                        format!("\n\nGenerated bootstrap code:\n{}", format_code_block(&bootstrap_result.source))
                    } else {
                        String::new()
                    };

                    // Generate rerun command using the script
                    let rerun_cmd = format!(
                        "scripts/glsl-filetests.sh {}:{}",
                        relative_path, directive.line_number
                    );

                    let error_msg = if let Some(state) = emulator_state {
                        format!(
                            "run test failed at line {}: {}{}{}{}\n\
                             \n\
                             This test assertion can be automatically updated by setting the\n\
                             CRANELIFT_TEST_BLESS=1 environment variable when running this test.\n\
                             \n\
                             To rerun just this test:\n\
                             {}",
                            directive.line_number, err_msg, bootstrap_code_display, clif_ir_section, state, rerun_cmd
                        )
                    } else {
                        format!(
                            "run test failed at line {}: {}{}{}\n\
                             \n\
                             This test assertion can be automatically updated by setting the\n\
                             CRANELIFT_TEST_BLESS=1 environment variable when running this test.\n\
                             \n\
                             To rerun just this test:\n\
                             {}",
                            directive.line_number, err_msg, bootstrap_code_display, clif_ir_section, rerun_cmd
                        )
                    };
                    anyhow::bail!("{}", error_msg);
                }
            }
        }
    }

    Ok(())
}

/// Format a compilation error with bootstrap code context
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

    // In compact mode, extract just the essential error (code + message) without notes
    let error_msg = if show_full_output {
        // Full mode: show everything including notes
        let full_error = error.to_string();
        let has_prefix = full_error.contains("Compilation error:");
        (full_error, has_prefix)
    } else {
        // Compact mode: show only error code and message, strip notes
        let basic_error = format!("error[{}]: {}", error.code, error.message);
        let has_prefix = basic_error.contains("Compilation error:");
        (basic_error, has_prefix)
    };

    // Extract notes if present (these contain detailed verifier errors) - only in full output mode
    let notes = if show_full_output && !error.notes.is_empty() {
        format!("\n\n{}", error.notes.join("\n"))
    } else {
        String::new()
    };

    // Format bootstrap code only when showing full output
    let bootstrap_section = if show_full_output {
        format!(
            "\n\nGenerated bootstrap code:\n{}",
            format_code_block(&bootstrap.source)
        )
    } else {
        String::new()
    };

    // Build the error message - simplified format for compact mode
    let mut msg = if show_full_output {
        format!(
            "Compilation failed for test case at line {}:\n\
             \n\
             Test case: {}{}\
             \n\
             {}{}{}",
            directive_line,
            expression,
            bootstrap_section,
            if error_msg.1 {
                ""
            } else {
                "Compilation error:\n"
            },
            error_msg.0,
            notes
        )
    } else {
        // Compact mode: just show the essential error
        format!(
            "Compilation failed for test case at line {}:\n\
             \n\
             Test case: {}\n\
             {}{}",
            directive_line,
            expression,
            if error_msg.1 {
                ""
            } else {
                "Compilation error:\n"
            },
            error_msg.0
        )
    };

    // Add main function span info for reference (only when showing full output)
    if show_full_output {
        msg.push_str(&format!(
            "\n\nNote: main() function spans lines {} to {}",
            bootstrap.main_start_line, bootstrap.main_end_line
        ));
    }

    // Add rerun command
    msg.push_str(&format!("\n\nTo rerun just this test:\n{}", rerun_cmd));

    anyhow::anyhow!("{}", msg)
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
