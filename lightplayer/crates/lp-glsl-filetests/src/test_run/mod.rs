//! Test execution and result comparison.

pub mod bootstrap;
pub mod execution;
pub mod target;
pub mod value_ops;

use crate::file_update::FileUpdate;
use crate::filetest::TestFile;
use anyhow::Result;
use lp_glsl::GlslOptions;
use lp_glsl::glsl_emu_riscv32;
use std::env;
use std::path::Path;

/// Run all tests in a test file.
pub fn run_test_file(test_file: &TestFile, path: &Path) -> Result<()> {
    if !test_file.is_test_run {
        // Not a test run file, skip
        return Ok(());
    }

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
        // Generate bootstrap code
        let bootstrap_source =
            bootstrap::generate_bootstrap(&test_file.glsl_source, &directive.expression_str)?;

        // Compile and execute
        // Note: bootstrap_source now contains ONLY the function being tested + main()
        let mut executable = glsl_emu_riscv32(&bootstrap_source, options.clone()).map_err(|e| {
            anyhow::anyhow!(
                "Compilation failed for test case at line {}:\n\
                     \n\
                     Test case: {}\n\
                     \n\
                     Generated bootstrap code:\n\
                     {}\n\
                     \n\
                     Compilation error:\n\
                     {}",
                directive.line_number,
                directive.expression_str,
                format_code_block(&bootstrap_source),
                e
            )
        })?;

        // Execute main() and get result
        let actual_value = execution::execute_main(&mut *executable)?;

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
                    // Get emulator state if available
                    let emulator_state = executable.format_emulator_state();
                    let error_msg = if let Some(state) = emulator_state {
                        format!(
                            "run test failed at line {}: {}{}\n\
                             This test assertion can be automatically updated by setting the\n\
                             CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                            directive.line_number, err_msg, state
                        )
                    } else {
                        format!(
                            "run test failed at line {}: {}\n\
                             This test assertion can be automatically updated by setting the\n\
                             CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                            directive.line_number, err_msg
                        )
                    };
                    anyhow::bail!("{}", error_msg);
                }
            }
        }
    }

    Ok(())
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
