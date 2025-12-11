//! Test execution and result comparison.

use crate::file_update::FileUpdate;
use crate::filetest::{ComparisonOp, TestFile};
use anyhow::{Context, Result};
use lp_glsl::semantic::types::Type;
use lp_glsl::{DecimalFormat, GlslExecutable, GlslOptions, GlslValue, RunMode, glsl_emu_riscv32};
use std::env;
use std::path::Path;

/// Run all tests in a test file.
pub fn run_test_file(test_file: &TestFile, path: &Path) -> Result<()> {
    if !test_file.is_test_run {
        // Not a test run file, skip
        return Ok(());
    }

    // Determine target and options
    let target = test_file.target.as_deref().unwrap_or("riscv32.fixed32");
    let (run_mode, decimal_format) = parse_target(target)?;

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
            generate_bootstrap(&test_file.glsl_source, &directive.expression_str)?;

        // Compile and execute
        let mut executable = glsl_emu_riscv32(&bootstrap_source, options.clone())
            .map_err(|e| {
                anyhow::anyhow!(
                    "failed to compile bootstrap for {}: {}",
                    directive.expression_str,
                    e
                )
            })
            .with_context(|| {
                format!(
                    "failed to compile bootstrap for {}",
                    directive.expression_str
                )
            })?;

        // Execute main() and get result
        let actual_value = execute_main(&mut *executable)?;

        // Parse expected value
        let expected_value = parse_glsl_value(&directive.expected_str)?;

        // Compare results
        match compare_results(&actual_value, &expected_value, directive.comparison) {
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

/// Generate bootstrap GLSL code that wraps the expression in a main() function.
fn generate_bootstrap(original_source: &str, expression_str: &str) -> Result<String> {
    // Infer return type by extracting function name from expression and looking it up in source
    let return_type = infer_return_type_from_expression(original_source, expression_str)?;

    let mut bootstrap = original_source.to_string();
    bootstrap.push_str(&format!(
        "\n{} main() {{\n    return {};\n}}\n",
        return_type, expression_str
    ));

    Ok(bootstrap)
}

/// Infer the return type of an expression by looking up the called function's return type.
fn infer_return_type_from_expression(source: &str, expression: &str) -> Result<String> {
    // Extract function name from expression (e.g., "add_float(0.0, 0.0)" -> "add_float")
    let func_name = expression
        .split('(')
        .next()
        .ok_or_else(|| anyhow::anyhow!("invalid expression: no opening parenthesis"))?
        .trim();

    // Search for function definition in source
    // Look for pattern: "return_type func_name("
    for line in source.lines() {
        let trimmed = line.trim();
        // Check if this line contains the function name followed by '('
        if let Some(pos) = trimmed.find(&format!("{}(", func_name)) {
            // Extract the part before the function name
            let before_func = &trimmed[..pos].trim();
            // The return type should be the last word before the function name
            if let Some(return_type) = before_func.split_whitespace().last() {
                // Validate it's a known type
                match return_type {
                    "float" | "int" | "bool" => return Ok(return_type.to_string()),
                    _ => {}
                }
            }
        }
    }

    // Fallback: use heuristic based on function name or expression content
    if func_name.contains("float") || expression.contains('.') {
        Ok("float".to_string())
    } else {
        Ok("int".to_string())
    }
}

/// Execute main() and return the result as a GlslValue.
fn execute_main(executable: &mut dyn GlslExecutable) -> Result<GlslValue> {
    // Try to get the signature to determine return type
    let sig = executable
        .get_function_signature("main")
        .ok_or_else(|| anyhow::anyhow!("main function not found"))?;

    // Call main() based on return type
    match &sig.return_type {
        Type::Float => {
            executable
                .call_f32("main", &[])
                .map(GlslValue::F32)
                .map_err(|e| {
                    // Add emulator state to error if available
                    if let Some(state) = executable.format_emulator_state() {
                        anyhow::anyhow!("{}{}", e, state)
                    } else {
                        anyhow::anyhow!("{}", e)
                    }
                })
        }
        Type::Int => {
            executable
                .call_i32("main", &[])
                .map(GlslValue::I32)
                .map_err(|e| {
                    // Add emulator state to error if available
                    if let Some(state) = executable.format_emulator_state() {
                        anyhow::anyhow!("{}{}", e, state)
                    } else {
                        anyhow::anyhow!("{}", e)
                    }
                })
        }
        Type::Bool => {
            executable
                .call_bool("main", &[])
                .map(GlslValue::Bool)
                .map_err(|e| {
                    // Add emulator state to error if available
                    if let Some(state) = executable.format_emulator_state() {
                        anyhow::anyhow!("{}{}", e, state)
                    } else {
                        anyhow::anyhow!("{}", e)
                    }
                })
        }
        _ => anyhow::bail!("unsupported return type: {:?}", sig.return_type),
    }
}

/// Parse a GLSL value from a string.
fn parse_glsl_value(s: &str) -> Result<GlslValue> {
    let s = s.trim();

    // Try parsing as integer
    if let Ok(i) = s.parse::<i32>() {
        return Ok(GlslValue::I32(i));
    }

    // Try parsing as float
    if let Ok(f) = s.parse::<f32>() {
        return Ok(GlslValue::F32(f));
    }

    // Try parsing as boolean
    match s {
        "true" => return Ok(GlslValue::Bool(true)),
        "false" => return Ok(GlslValue::Bool(false)),
        _ => {}
    }

    anyhow::bail!("failed to parse GLSL value: {}", s)
}

/// Compare actual and expected values.
fn compare_results(
    actual: &GlslValue,
    expected: &GlslValue,
    comparison: ComparisonOp,
) -> Result<(), String> {
    match (actual, expected) {
        (GlslValue::I32(a), GlslValue::I32(e)) => {
            if comparison == ComparisonOp::Exact && a == e {
                Ok(())
            } else if comparison == ComparisonOp::Exact {
                Err(format!("expected {}, got {}", e, a))
            } else {
                Err(format!("exact comparison required for integers, got ~="))
            }
        }
        (GlslValue::F32(a), GlslValue::F32(e)) => {
            let tolerance = 1e-4; // Default tolerance for float comparisons
            let diff = (a - e).abs();
            if diff <= tolerance {
                Ok(())
            } else {
                Err(format!(
                    "expected {} (tolerance: {}), got {} (diff: {})",
                    e, tolerance, a, diff
                ))
            }
        }
        (GlslValue::Bool(a), GlslValue::Bool(e)) => {
            if comparison == ComparisonOp::Exact && a == e {
                Ok(())
            } else if comparison == ComparisonOp::Exact {
                Err(format!("expected {}, got {}", e, a))
            } else {
                Err(format!("exact comparison required for booleans, got ~="))
            }
        }
        _ => Err(format!(
            "type mismatch: expected {:?}, got {:?}",
            expected, actual
        )),
    }
}

/// Parse target string (e.g., "riscv32.fixed32") into run mode and decimal format.
fn parse_target(target: &str) -> Result<(RunMode, DecimalFormat)> {
    let parts: Vec<&str> = target.split('.').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "invalid target format: expected '<arch>.<format>', got '{}'",
            target
        );
    }

    let arch = parts[0];
    let format = parts[1];

    let run_mode = match arch {
        "riscv32" => RunMode::Emulator {
            max_memory: 1024 * 1024, // 1MB
            stack_size: 64 * 1024,   // 64KB
            max_instructions: 10_000_000,
        },
        _ => anyhow::bail!("unsupported architecture: {}", arch),
    };

    let decimal_format = match format {
        "fixed32" => DecimalFormat::Fixed32,
        "float" => DecimalFormat::Float,
        _ => anyhow::bail!("unsupported format: {}", format),
    };

    Ok((run_mode, decimal_format))
}
