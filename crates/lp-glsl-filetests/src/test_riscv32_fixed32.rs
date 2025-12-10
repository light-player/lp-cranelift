//! Test riscv32.fixed32 with dual CLIF output
//! Generates both Generated CLIF (before transformation) and Transformed CLIF (after transformation)

use anyhow::{Result, bail};
use regex::Regex;
use std::path::Path;

use crate::execution::backend::ReturnType;
use crate::execution::binary::compile_to_binary;
use crate::execution::{CompiledCode, EmulatorBackend, EmulatorType};
use crate::filetest::TestTarget;
use crate::filetest::build_isa_for_target;
use crate::test_run::{ExpectedType, parse_run_directives};

/// Run riscv32.fixed32 test with dual CLIF output
pub fn run_test(path: &Path, full_source: &str, glsl_source: &str) -> Result<()> {
    // Build riscv32 ISA
    let isa = build_isa_for_target(TestTarget::Riscv32(Some(
        lp_glsl::FixedPointFormat::Fixed16x16,
    )))?;

    // Create JIT with riscv32 ISA and Fixed16x16 format
    let mut jit = lp_glsl::JIT::new_with_isa(isa);
    jit.fixed_point_format = Some(lp_glsl::FixedPointFormat::Fixed16x16);

    // Generate CLIF BEFORE transformation
    let generated_clif = jit
        .compile_to_clif_detailed(glsl_source, false)
        .map_err(|e| anyhow::anyhow!("Failed to generate CLIF (before transformation): {}", e))?;

    // Generate CLIF AFTER transformation
    let transformed_clif = jit
        .compile_to_clif_detailed(glsl_source, true)
        .map_err(|e| anyhow::anyhow!("Failed to generate CLIF (after transformation): {}", e))?;

    // Extract expected CLIF sections
    let (expected_generated, expected_transformed) = extract_dual_clif_expectations(full_source);

    // Normalize architecture-specific calling convention names
    let generated_normalized = normalize_architecture_names(generated_clif.trim());
    let transformed_normalized = normalize_architecture_names(transformed_clif.trim());
    let expected_generated_normalized = normalize_architecture_names(expected_generated.trim());
    let expected_transformed_normalized = normalize_architecture_names(expected_transformed.trim());

    // Compare CLIF outputs
    let generated_match = generated_normalized == expected_generated_normalized;
    let transformed_match = transformed_normalized == expected_transformed_normalized;

    // Handle BLESS mode
    if crate::file_update::is_bless_enabled() {
        if !generated_match || !transformed_match {
            crate::file_update::update_riscv32_fixed32_expectations(
                path,
                &generated_clif,
                &transformed_clif,
            )?;
            return Ok(());
        }
    }

    // Report mismatches
    if !generated_match || !transformed_match {
        let mut error_msg = String::from("CLIF output mismatch.\n\n");

        if !generated_match {
            error_msg.push_str("Generated CLIF (before transformation) mismatch:\n");
            error_msg.push_str("Expected (normalized):\n");
            error_msg.push_str(&expected_generated_normalized);
            error_msg.push_str("\n\nActual (normalized):\n");
            error_msg.push_str(&generated_normalized);
            error_msg.push_str("\n\n");
        }

        if !transformed_match {
            error_msg.push_str("Transformed CLIF (after transformation) mismatch:\n");
            error_msg.push_str("Expected (normalized):\n");
            error_msg.push_str(&expected_transformed_normalized);
            error_msg.push_str("\n\nActual (normalized):\n");
            error_msg.push_str(&transformed_normalized);
            error_msg.push_str("\n\n");
        }

        error_msg.push_str("This test assertion can be automatically updated by setting the\n");
        error_msg.push_str("CRANELIFT_TEST_BLESS=1 environment variable when running this test.");

        bail!("{}", error_msg);
    }

    // Run test in riscv32 emulator
    let run_directives = parse_run_directives(full_source)?;
    if run_directives.is_empty() {
        // No run directives, just verify CLIF
        return Ok(());
    }

    // Determine return type from first run directive
    let return_type = match &run_directives[0].expected_type {
        ExpectedType::Vec2Approx { .. } => ReturnType::Vec2,
        ExpectedType::Vec3Approx { .. } => ReturnType::Vec3,
        ExpectedType::Vec4Approx { .. } => ReturnType::Vec4,
        ExpectedType::Mat2Approx { .. } => ReturnType::Mat2,
        ExpectedType::Mat3Approx { .. } => ReturnType::Mat3,
        ExpectedType::Mat4Approx { .. } => ReturnType::Mat4,
        ExpectedType::FloatApprox { .. } => ReturnType::Float,
        ExpectedType::Int(_) => ReturnType::Int,
        ExpectedType::Bool(_) => ReturnType::Bool,
    };
    
    let fixed_point_format = Some(lp_glsl::FixedPointFormat::Fixed16x16);

    // Compile to binary
    let binary = compile_to_binary(
        glsl_source,
        Some(lp_glsl::FixedPointFormat::Fixed16x16),
        return_type,
    )?;

    let compiled_code = CompiledCode::EmulatorBinary {
        binary,
        return_type,
    };

    // Execute using emulator backend
    let backend = EmulatorBackend::new(EmulatorType::Riscv32);

    // Execute and verify results
    // For fixed-point, we need to read i32 values and convert to f32
    for directive in &run_directives {
        match &directive.expected_type {
            ExpectedType::FloatApprox { expected } => {
                // Scalar float: read i32 from memory and convert to f32
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let val = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let result_float = val as f32 / 65536.0;
                let tolerance = 0.001;
                let diff = (result_float - expected).abs();
                if diff > tolerance {
                    bail!(
                        "Run test failed: expected {} (tolerance {}), got {} (diff: {})",
                        expected, tolerance, result_float, diff
                    );
                }
            }
            ExpectedType::Vec2Approx { expected } => {
                // Read i32 values from memory and convert to f32
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let val0 = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let val1 = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let result_vec = [val0 as f32 / 65536.0, val1 as f32 / 65536.0];
                let tolerance = 0.001; // Fixed16x16 tolerance
                let mut max_diff = 0.0f32;
                for i in 0..2 {
                    let diff = (result_vec[i] - expected[i]).abs();
                    max_diff = max_diff.max(diff);
                }
                if max_diff > tolerance {
                    bail!(
                        "Run test failed: expected vec2({}, {}) (tolerance {}), got vec2({}, {}) (max diff: {})",
                        expected[0],
                        expected[1],
                        tolerance,
                        result_vec[0],
                        result_vec[1],
                        max_diff
                    );
                }
            }
            ExpectedType::Vec3Approx { expected } => {
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let val0 = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let val1 = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let val2 = memory
                    .read_word(RESULT_ADDR + 8)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let result_vec = [
                    val0 as f32 / 65536.0,
                    val1 as f32 / 65536.0,
                    val2 as f32 / 65536.0,
                ];
                let tolerance = 0.001;
                let mut max_diff = 0.0f32;
                for i in 0..3 {
                    let diff = (result_vec[i] - expected[i]).abs();
                    max_diff = max_diff.max(diff);
                }
                if max_diff > tolerance {
                    bail!(
                        "Run test failed: expected vec3({}, {}, {}) (tolerance {}), got vec3({}, {}, {}) (max diff: {})",
                        expected[0],
                        expected[1],
                        expected[2],
                        tolerance,
                        result_vec[0],
                        result_vec[1],
                        result_vec[2],
                        max_diff
                    );
                }
            }
            ExpectedType::Vec4Approx { expected } => {
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let val0 = memory
                    .read_word(RESULT_ADDR)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let val1 = memory
                    .read_word(RESULT_ADDR + 4)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let val2 = memory
                    .read_word(RESULT_ADDR + 8)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let val3 = memory
                    .read_word(RESULT_ADDR + 12)
                    .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                let result_vec = [
                    val0 as f32 / 65536.0,
                    val1 as f32 / 65536.0,
                    val2 as f32 / 65536.0,
                    val3 as f32 / 65536.0,
                ];
                let tolerance = 0.001;
                let mut max_diff = 0.0f32;
                for i in 0..4 {
                    let diff = (result_vec[i] - expected[i]).abs();
                    max_diff = max_diff.max(diff);
                }
                if max_diff > tolerance {
                    bail!(
                        "Run test failed: expected vec4({}, {}, {}, {}) (tolerance {}), got vec4({}, {}, {}, {}) (max diff: {})",
                        expected[0],
                        expected[1],
                        expected[2],
                        expected[3],
                        tolerance,
                        result_vec[0],
                        result_vec[1],
                        result_vec[2],
                        result_vec[3],
                        max_diff
                    );
                }
            }
            ExpectedType::Mat2Approx { expected } => {
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let mut result_mat = [0.0f32; 4];
                for i in 0..4 {
                    let val = memory
                        .read_word(RESULT_ADDR + (i * 4) as u32)
                        .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                    result_mat[i] = val as f32 / 65536.0;
                }
                let tolerance = 0.001;
                let mut max_diff = 0.0f32;
                for i in 0..4 {
                    let diff = (result_mat[i] - expected[i]).abs();
                    max_diff = max_diff.max(diff);
                }
                if max_diff > tolerance {
                    bail!(
                        "Run test failed: expected mat2({}, {}, {}, {}) (tolerance {}), got mat2({}, {}, {}, {}) (max diff: {})",
                        expected[0], expected[1], expected[2], expected[3],
                        tolerance,
                        result_mat[0], result_mat[1], result_mat[2], result_mat[3],
                        max_diff
                    );
                }
            }
            ExpectedType::Mat3Approx { expected } => {
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let mut result_mat = [0.0f32; 9];
                for i in 0..9 {
                    let val = memory
                        .read_word(RESULT_ADDR + (i * 4) as u32)
                        .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                    result_mat[i] = val as f32 / 65536.0;
                }
                let tolerance = 0.001;
                let mut max_diff = 0.0f32;
                for i in 0..9 {
                    let diff = (result_mat[i] - expected[i]).abs();
                    max_diff = max_diff.max(diff);
                }
                if max_diff > tolerance {
                    bail!(
                        "Run test failed: expected mat3({}, {}, {}, {}, {}, {}, {}, {}, {}) (tolerance {}), got mat3({}, {}, {}, {}, {}, {}, {}, {}, {}) (max diff: {})",
                        expected[0], expected[1], expected[2], expected[3], expected[4],
                        expected[5], expected[6], expected[7], expected[8],
                        tolerance,
                        result_mat[0], result_mat[1], result_mat[2], result_mat[3], result_mat[4],
                        result_mat[5], result_mat[6], result_mat[7], result_mat[8],
                        max_diff
                    );
                }
            }
            ExpectedType::Mat4Approx { expected } => {
                let emu = match &compiled_code {
                    CompiledCode::EmulatorBinary { binary, .. } => backend.run_emulator(binary)?,
                    _ => bail!("EmulatorBackend requires EmulatorBinary compiled code"),
                };
                let memory = emu.memory();
                use crate::execution::bootstrap::RESULT_ADDR;
                let mut result_mat = [0.0f32; 16];
                for i in 0..16 {
                    let val = memory
                        .read_word(RESULT_ADDR + (i * 4) as u32)
                        .map_err(|e| anyhow::anyhow!("Failed to read result from memory: {:?}", e))?;
                    result_mat[i] = val as f32 / 65536.0;
                }
                let tolerance = 0.001;
                let mut max_diff = 0.0f32;
                for i in 0..16 {
                    let diff = (result_mat[i] - expected[i]).abs();
                    max_diff = max_diff.max(diff);
                }
                if max_diff > tolerance {
                    bail!(
                        "Run test failed: expected mat4 (tolerance {}), got mat4 (max diff: {})",
                        tolerance, max_diff
                    );
                }
            }
            _ => {
                bail!("Unsupported return type for riscv32.fixed32 test");
            }
        }
    }

    Ok(())
}

/// Extract expected CLIF sections from test file
fn extract_dual_clif_expectations(source: &str) -> (String, String) {
    let mut generated_lines = Vec::new();
    let mut transformed_lines = Vec::new();
    let mut in_generated = false;
    let mut in_transformed = false;

    for line in source.lines() {
        let trimmed = line.trim();

        // Check for section markers
        if let Some(comment_content) = trimmed.strip_prefix("//") {
            let content = comment_content.trim();

            // Skip test directives and other special comments
            if content.starts_with("test ")
                || content.starts_with("target")
                || content.starts_with("CHECK")
                || content.starts_with("run:")
                || content.starts_with("EXPECT_ERROR:")
                || content.starts_with("Validate")
            {
                continue;
            }

            // Check for section markers
            if content == "Generated CLIF" {
                in_generated = true;
                in_transformed = false;
                continue;
            }
            if content == "Transformed CLIF" {
                in_transformed = true;
                in_generated = false;
                continue;
            }

            // Collect lines based on current section
            if in_generated {
                // Stop if we hit Transformed CLIF marker
                if content == "Transformed CLIF" {
                    in_generated = false;
                    in_transformed = true;
                    continue;
                }
                // Stop if we hit run directive
                if content.starts_with("run:") {
                    break;
                }
                generated_lines.push(content.to_string());
            } else if in_transformed {
                // Stop if we hit run directive
                if content.starts_with("run:") {
                    break;
                }
                transformed_lines.push(content.to_string());
            } else {
                // Only start collecting when we see CLIF-like patterns
                if content.starts_with("function") || content.starts_with("block") {
                    // This might be the start of Generated CLIF section
                    // But we haven't seen the marker yet, so skip
                    continue;
                }
            }
        } else if (in_generated || in_transformed) && !trimmed.is_empty() {
            // Non-comment line after starting a section, stop
            break;
        }
    }

    (generated_lines.join("\n"), transformed_lines.join("\n"))
}

/// Normalize architecture-specific calling convention names in CLIF IR output
/// Also normalizes indentation by removing leading whitespace from each line
fn normalize_architecture_names(clif_output: &str) -> String {
    let re = Regex::new(r"\s+(fast|cold|tail|system_v|windows_fastcall|apple_aarch64|probestack|winch|patchable)\s*\{")
        .expect("Failed to compile regex pattern");

    let normalized = re.replace_all(clif_output, " ARCH {").to_string();

    // Normalize indentation: remove leading whitespace from each line
    normalized
        .lines()
        .map(|line| line.trim_start())
        .collect::<Vec<_>>()
        .join("\n")
}
