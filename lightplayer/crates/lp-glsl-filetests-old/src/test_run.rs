//! Test JIT execution with result verification
//! Pattern: cranelift/filetests/src/test_run.rs

use anyhow::Result;
use std::path::Path;

use crate::execution::backend::ReturnType;
use crate::filetest::TestTarget;

/// Get tolerance default based on target
fn get_tolerance_default(target: &TestTarget) -> f32 {
    match target {
        TestTarget::Host(None) => 0.0001,
        TestTarget::Host(Some(lp_glsl::FixedPointFormat::Fixed16x16)) => 0.001,
        TestTarget::Host(Some(lp_glsl::FixedPointFormat::Fixed32x32)) => 0.0001,
        TestTarget::Riscv32(None) => 0.0001,
        TestTarget::Riscv32(Some(lp_glsl::FixedPointFormat::Fixed16x16)) => 0.001,
        TestTarget::Riscv32(Some(lp_glsl::FixedPointFormat::Fixed32x32)) => 0.0001,
    }
}

/// Helper function to compare approximate float arrays and handle bless mode
fn compare_approx_array<const N: usize>(
    path: &Path,
    expected: &[f32; N],
    actual: &[f32; N],
    tolerance: f32,
    _type_name: &str,
    format_bless: impl Fn(&[f32; N], f32) -> String,
    format_error: impl Fn(&[f32; N], &[f32; N], f32, f32) -> String,
) -> Result<()> {
    let mut max_diff = 0.0f32;
    for i in 0..N {
        let diff = (actual[i] - expected[i]).abs();
        max_diff = max_diff.max(diff);
    }

    if max_diff > tolerance {
        if crate::file_update::is_bless_enabled() {
            let new_directive = format_bless(actual, tolerance);
            crate::file_update::update_run_directive(path, &new_directive)?;
            return Ok(());
        }

        anyhow::bail!(
            "{}\n\
             \n\
             This test assertion can be automatically updated by setting the\n\
             CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
            format_error(expected, actual, tolerance, max_diff)
        );
    }

    Ok(())
}

pub fn run_test(
    path: &Path,
    _full_source: &str,
    glsl_source: &str,
    targets: &[TestTarget],
) -> Result<()> {
    // Parse run directives: // run: <expected_result>
    let run_directives = parse_run_directives(_full_source)?;

    if run_directives.is_empty() {
        anyhow::bail!("No 'run' directives found");
    }

    // Execute test for each target
    for target in targets {
        // Compile and execute for this target
        for directive in &run_directives {
            match directive.expected_type {
                ExpectedType::Int(expected) => {
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            None, // Int tests don't use fixed-point format
                            ReturnType::Int,
                            |backend, code, _| backend.execute_int(code, None),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        let func = compiler.compile_int(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    if result != expected {
                        // If BLESS mode is enabled, update the test file
                        if crate::file_update::is_bless_enabled() {
                            let new_directive = format!("== {}", result);
                            crate::file_update::update_run_directive(path, &new_directive)?;
                            return Ok(());
                        }

                        anyhow::bail!(
                            "Run test failed: expected {}, got {}\n\
                         \n\
                         This test assertion can be automatically updated by setting the\n\
                         CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                            expected,
                            result
                        );
                    }
                }
                ExpectedType::Bool(expected) => {
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Bool,
                            |backend, code, fmt| backend.execute_bool(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_bool(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };
                    let expected_val = if expected { 1 } else { 0 };

                    if result != expected_val {
                        // If BLESS mode is enabled, update the test file
                        if crate::file_update::is_bless_enabled() {
                            let new_directive = format!("== {}", result != 0);
                            crate::file_update::update_run_directive(path, &new_directive)?;
                            return Ok(());
                        }

                        anyhow::bail!(
                            "Run test failed: expected {}, got {}\n\
                         \n\
                         This test assertion can be automatically updated by setting the\n\
                         CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                            expected,
                            result != 0
                        );
                    }
                }
                ExpectedType::FloatApprox { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();

                    let result_float = if target.is_riscv32() {
                        // Use riscv32 emulator
                        execute_riscv32_float(glsl_source, fixed_point_format)?
                    } else {
                        // Use native JIT
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);

                        if let Some(format) = fixed_point_format {
                            match format {
                                lp_glsl::FixedPointFormat::Fixed16x16 => {
                                    let func = compiler.compile_int(glsl_source).map_err(|e| {
                                        anyhow::anyhow!("Failed to compile for run test: {}", e)
                                    })?;
                                    func() as f32 / 65536.0
                                }
                                lp_glsl::FixedPointFormat::Fixed32x32 => {
                                    let func = compiler.compile_i64(glsl_source).map_err(|e| {
                                        anyhow::anyhow!("Failed to compile for run test: {}", e)
                                    })?;
                                    (func() as f64 / 4294967296.0) as f32
                                }
                            }
                        } else {
                            let func = compiler.compile_float(glsl_source).map_err(|e| {
                                anyhow::anyhow!("Failed to compile for run test: {}", e)
                            })?;
                            func()
                        }
                    };

                    let diff = (result_float - expected).abs();
                    if diff > tolerance {
                        // If BLESS mode is enabled, update the test file
                        if crate::file_update::is_bless_enabled() {
                            let new_directive = format!("~= {}", result_float);
                            crate::file_update::update_run_directive(path, &new_directive)?;
                            return Ok(());
                        }

                        anyhow::bail!(
                            "Run test failed: expected {} (tolerance {}), got {} (diff: {})\n\
                         \n\
                         This test assertion can be automatically updated by setting the\n\
                         CRANELIFT_TEST_BLESS=1 environment variable when running this test.",
                            expected,
                            tolerance,
                            result_float,
                            diff
                        );
                    }
                }
                ExpectedType::Vec2Approx { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Vec2,
                            |backend, code, fmt| backend.execute_vec2(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_vec2(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    let result_vec = [result.0, result.1];
                    compare_approx_array(
                        path,
                        &expected,
                        &result_vec,
                        tolerance,
                        "vec2",
                        |actual, _| format!("≈ vec2({}, {})", actual[0], actual[1]),
                        |expected, actual, tol, max_diff| {
                            format!(
                                "Run test failed: expected vec2({}, {}) (tolerance {}), got vec2({}, {}) (max diff: {})",
                                expected[0], expected[1], tol, actual[0], actual[1], max_diff
                            )
                        },
                    )?;
                }
                ExpectedType::Vec3Approx { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Vec3,
                            |backend, code, fmt| backend.execute_vec3(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_vec3(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    let result_vec = [result.0, result.1, result.2];
                    compare_approx_array(
                        path,
                        &expected,
                        &result_vec,
                        tolerance,
                        "vec3",
                        |actual, _tol| {
                            format!("≈ vec3({}, {}, {})", actual[0], actual[1], actual[2])
                        },
                        |expected, actual, tol, max_diff| {
                            format!(
                                "Run test failed: expected vec3({}, {}, {}) (tolerance {}), got vec3({}, {}, {}) (max diff: {})",
                                expected[0],
                                expected[1],
                                expected[2],
                                tol,
                                actual[0],
                                actual[1],
                                actual[2],
                                max_diff
                            )
                        },
                    )?;
                }
                ExpectedType::Vec4Approx { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Vec4,
                            |backend, code, fmt| backend.execute_vec4(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_vec4(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    let result_vec = [result.0, result.1, result.2, result.3];
                    compare_approx_array(
                        path,
                        &expected,
                        &result_vec,
                        tolerance,
                        "vec4",
                        |actual, _tol| {
                            format!(
                                "≈ vec4({}, {}, {}, {})",
                                actual[0], actual[1], actual[2], actual[3]
                            )
                        },
                        |expected, actual, tol, max_diff| {
                            format!(
                                "Run test failed: expected vec4({}, {}, {}, {}) (tolerance {}), got vec4({}, {}, {}, {}) (max diff: {})",
                                expected[0],
                                expected[1],
                                expected[2],
                                expected[3],
                                tol,
                                actual[0],
                                actual[1],
                                actual[2],
                                actual[3],
                                max_diff
                            )
                        },
                    )?;
                }
                ExpectedType::Mat2Approx { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Mat2,
                            |backend, code, fmt| backend.execute_mat2(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_mat2(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    let result_mat = [result.0, result.1, result.2, result.3];
                    compare_approx_array(
                        path,
                        &expected,
                        &result_mat,
                        tolerance,
                        "mat2",
                        |actual, _tol| {
                            format!(
                                "≈ mat2({}, {}, {}, {})",
                                actual[0], actual[1], actual[2], actual[3]
                            )
                        },
                        |expected, actual, tol, max_diff| {
                            format!(
                                "Run test failed: expected mat2({}, {}, {}, {}) (tolerance {}), got mat2({}, {}, {}, {}) (max diff: {})",
                                expected[0],
                                expected[1],
                                expected[2],
                                expected[3],
                                tol,
                                actual[0],
                                actual[1],
                                actual[2],
                                actual[3],
                                max_diff
                            )
                        },
                    )?;
                }
                ExpectedType::Mat3Approx { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Mat3,
                            |backend, code, fmt| backend.execute_mat3(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_mat3(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    let result_mat = [
                        result.0, result.1, result.2, result.3, result.4, result.5, result.6,
                        result.7, result.8,
                    ];
                    compare_approx_array(
                        path,
                        &expected,
                        &result_mat,
                        tolerance,
                        "mat3",
                        |actual, _tol| {
                            format!(
                                "≈ mat3({}, {}, {}, {}, {}, {}, {}, {}, {})",
                                actual[0],
                                actual[1],
                                actual[2],
                                actual[3],
                                actual[4],
                                actual[5],
                                actual[6],
                                actual[7],
                                actual[8]
                            )
                        },
                        |expected, actual, tol, max_diff| {
                            format!(
                                "Run test failed: expected mat3({}, {}, {}, {}, {}, {}, {}, {}, {}) (tolerance {}), got mat3({}, {}, {}, {}, {}, {}, {}, {}, {}) (max diff: {})",
                                expected[0],
                                expected[1],
                                expected[2],
                                expected[3],
                                expected[4],
                                expected[5],
                                expected[6],
                                expected[7],
                                expected[8],
                                tol,
                                actual[0],
                                actual[1],
                                actual[2],
                                actual[3],
                                actual[4],
                                actual[5],
                                actual[6],
                                actual[7],
                                actual[8],
                                max_diff
                            )
                        },
                    )?;
                }
                ExpectedType::Mat4Approx { expected } => {
                    let tolerance = get_tolerance_default(target);
                    let fixed_point_format = target.fixed_point_format();
                    let result = if target.is_riscv32() {
                        use crate::execution::ExecutionBackend;
                        execute_riscv32(
                            glsl_source,
                            fixed_point_format,
                            ReturnType::Mat4,
                            |backend, code, fmt| backend.execute_mat4(code, fmt),
                        )?
                    } else {
                        let mut compiler = lp_glsl::Compiler::new();
                        compiler.set_fixed_point_format(fixed_point_format);
                        let func = compiler.compile_mat4(glsl_source).map_err(|e| {
                            anyhow::anyhow!("Failed to compile for run test: {}", e)
                        })?;
                        func()
                    };

                    let result_mat = [
                        result.0, result.1, result.2, result.3, result.4, result.5, result.6,
                        result.7, result.8, result.9, result.10, result.11, result.12, result.13,
                        result.14, result.15,
                    ];
                    compare_approx_array(
                        path,
                        &expected,
                        &result_mat,
                        tolerance,
                        "mat4",
                        |actual, _tol| {
                            format!(
                                "≈ mat4({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {})",
                                actual[0],
                                actual[1],
                                actual[2],
                                actual[3],
                                actual[4],
                                actual[5],
                                actual[6],
                                actual[7],
                                actual[8],
                                actual[9],
                                actual[10],
                                actual[11],
                                actual[12],
                                actual[13],
                                actual[14],
                                actual[15]
                            )
                        },
                        |expected, actual, tol, max_diff| {
                            format!(
                                "Run test failed: expected mat4({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) (tolerance {}), got mat4({}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}, {}) (max diff: {})",
                                expected[0],
                                expected[1],
                                expected[2],
                                expected[3],
                                expected[4],
                                expected[5],
                                expected[6],
                                expected[7],
                                expected[8],
                                expected[9],
                                expected[10],
                                expected[11],
                                expected[12],
                                expected[13],
                                expected[14],
                                expected[15],
                                tol,
                                actual[0],
                                actual[1],
                                actual[2],
                                actual[3],
                                actual[4],
                                actual[5],
                                actual[6],
                                actual[7],
                                actual[8],
                                actual[9],
                                actual[10],
                                actual[11],
                                actual[12],
                                actual[13],
                                actual[14],
                                actual[15],
                                max_diff
                            )
                        },
                    )?;
                }
            }
        }
    }

    Ok(())
}

pub struct RunDirective {
    pub expected_type: ExpectedType,
}

pub enum ExpectedType {
    Int(i32),
    Bool(bool),
    FloatApprox { expected: f32 },
    Vec2Approx { expected: [f32; 2] },
    Vec3Approx { expected: [f32; 3] },
    Vec4Approx { expected: [f32; 4] },
    Mat2Approx { expected: [f32; 4] },
    Mat3Approx { expected: [f32; 9] },
    Mat4Approx { expected: [f32; 16] },
}

pub fn parse_run_directives(source: &str) -> Result<Vec<RunDirective>> {
    let mut directives = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();
        // Parse // run: directives
        if let Some(comment) = trimmed.strip_prefix("//") {
            if let Some(run_spec) = comment.trim().strip_prefix("run:") {
                // Strip inline comments (everything after //)
                let spec = run_spec.split("//").next().unwrap_or(run_spec).trim();

                // Parse "≈ vecN(...)" for approximate vector comparison (tolerance removed - uses target defaults)
                if let Some(approx_str) = spec.strip_prefix("≈").map(str::trim) {
                    // Try to parse as vector
                    if let Some(vec_str) = approx_str.strip_prefix("vec2(") {
                        // Strip tolerance suffix if present: "1.5, 2.3) (tolerance: 0.01" -> "1.5, 2.3"
                        let values_str = if let Some(idx) = vec_str.find(") (tolerance:") {
                            &vec_str[..idx]
                        } else if vec_str.ends_with(')') {
                            &vec_str[..vec_str.len() - 1]
                        } else {
                            vec_str
                        };
                        let values: Vec<f32> = values_str
                            .split(',')
                            .map(|s| s.trim().parse::<f32>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|_| {
                                anyhow::anyhow!("Failed to parse vec2 values: {}", values_str)
                            })?;
                        if values.len() != 2 {
                            anyhow::bail!("vec2 expects 2 values, got {}", values.len());
                        }
                        directives.push(RunDirective {
                            expected_type: ExpectedType::Vec2Approx {
                                expected: [values[0], values[1]],
                            },
                        });
                        continue;
                    } else if let Some(vec_str) = approx_str.strip_prefix("vec3(") {
                        // Strip tolerance suffix if present: "4, -2, 1) (tolerance: 0.01" -> "4, -2, 1"
                        let values_str = if let Some(idx) = vec_str.find(") (tolerance:") {
                            &vec_str[..idx]
                        } else if vec_str.ends_with(')') {
                            &vec_str[..vec_str.len() - 1]
                        } else {
                            vec_str
                        };
                        let values: Vec<f32> = values_str
                            .split(',')
                            .map(|s| s.trim().parse::<f32>())
                            .collect::<Result<Vec<_>, _>>()
                            .map_err(|_| {
                                anyhow::anyhow!("Failed to parse vec3 values: {}", values_str)
                            })?;
                        if values.len() != 3 {
                            anyhow::bail!("vec3 expects 3 values, got {}", values.len());
                        }
                        directives.push(RunDirective {
                            expected_type: ExpectedType::Vec3Approx {
                                expected: [values[0], values[1], values[2]],
                            },
                        });
                        continue;
                    } else if let Some(vec_str) = approx_str.strip_prefix("vec4(") {
                        if let Some(values_str) = vec_str.strip_suffix(')') {
                            let values: Vec<f32> = values_str
                                .split(',')
                                .map(|s| s.trim().parse::<f32>())
                                .collect::<Result<Vec<_>, _>>()
                                .map_err(|_| {
                                    anyhow::anyhow!("Failed to parse vec4 values: {}", values_str)
                                })?;
                            if values.len() != 4 {
                                anyhow::bail!("vec4 expects 4 values, got {}", values.len());
                            }
                            directives.push(RunDirective {
                                expected_type: ExpectedType::Vec4Approx {
                                    expected: [values[0], values[1], values[2], values[3]],
                                },
                            });
                            continue;
                        }
                    } else if let Some(mat_str) = approx_str.strip_prefix("mat2(") {
                        if let Some(values_str) = mat_str.strip_suffix(')') {
                            let values: Vec<f32> = values_str
                                .split(',')
                                .map(|s| s.trim().parse::<f32>())
                                .collect::<Result<Vec<_>, _>>()
                                .map_err(|_| {
                                    anyhow::anyhow!("Failed to parse mat2 values: {}", values_str)
                                })?;
                            if values.len() != 4 {
                                anyhow::bail!("mat2 expects 4 values, got {}", values.len());
                            }
                            directives.push(RunDirective {
                                expected_type: ExpectedType::Mat2Approx {
                                    expected: [values[0], values[1], values[2], values[3]],
                                },
                            });
                            continue;
                        }
                    } else if let Some(mat_str) = approx_str.strip_prefix("mat3(") {
                        if let Some(values_str) = mat_str.strip_suffix(')') {
                            let values: Vec<f32> = values_str
                                .split(',')
                                .map(|s| s.trim().parse::<f32>())
                                .collect::<Result<Vec<_>, _>>()
                                .map_err(|_| {
                                    anyhow::anyhow!("Failed to parse mat3 values: {}", values_str)
                                })?;
                            if values.len() != 9 {
                                anyhow::bail!("mat3 expects 9 values, got {}", values.len());
                            }
                            directives.push(RunDirective {
                                expected_type: ExpectedType::Mat3Approx {
                                    expected: [
                                        values[0], values[1], values[2], values[3], values[4],
                                        values[5], values[6], values[7], values[8],
                                    ],
                                },
                            });
                            continue;
                        }
                    } else if let Some(mat_str) = approx_str.strip_prefix("mat4(") {
                        if let Some(values_str) = mat_str.strip_suffix(')') {
                            let values: Vec<f32> = values_str
                                .split(',')
                                .map(|s| s.trim().parse::<f32>())
                                .collect::<Result<Vec<_>, _>>()
                                .map_err(|_| {
                                    anyhow::anyhow!("Failed to parse mat4 values: {}", values_str)
                                })?;
                            if values.len() != 16 {
                                anyhow::bail!("mat4 expects 16 values, got {}", values.len());
                            }
                            directives.push(RunDirective {
                                expected_type: ExpectedType::Mat4Approx {
                                    expected: [
                                        values[0], values[1], values[2], values[3], values[4],
                                        values[5], values[6], values[7], values[8], values[9],
                                        values[10], values[11], values[12], values[13], values[14],
                                        values[15],
                                    ],
                                },
                            });
                            continue;
                        }
                    }
                }
                // Parse "~= <value>" for approximate float comparison (tolerance removed - uses target defaults)
                // Support both "~= <value>" and "~= <value> (tolerance: <tol>)" formats
                if let Some(value_str) = spec.strip_prefix("~=").map(str::trim) {
                    // Strip tolerance suffix if present: "4.0 (tolerance: 0.0001)" -> "4.0"
                    let value_str = if let Some(idx) = value_str.find(" (tolerance:") {
                        &value_str[..idx]
                    } else {
                        value_str
                    };
                    let value = value_str.parse::<f32>().map_err(|_| {
                        anyhow::anyhow!("Failed to parse float value: {}", value_str)
                    })?;
                    directives.push(RunDirective {
                        expected_type: ExpectedType::FloatApprox { expected: value },
                    });
                }
                // Parse "== <value>"
                else if let Some(expected_str) = spec.strip_prefix("==").map(str::trim) {
                    // Try parsing as int
                    if let Ok(val) = expected_str.parse::<i32>() {
                        directives.push(RunDirective {
                            expected_type: ExpectedType::Int(val),
                        });
                    }
                    // Try parsing as bool
                    else if expected_str == "true" {
                        directives.push(RunDirective {
                            expected_type: ExpectedType::Bool(true),
                        });
                    } else if expected_str == "false" {
                        directives.push(RunDirective {
                            expected_type: ExpectedType::Bool(false),
                        });
                    }
                }
            }
        }
    }

    Ok(directives)
}

/// Execute GLSL code in riscv32 emulator for a given return type
fn execute_riscv32<T>(
    glsl_source: &str,
    fixed_point_format: Option<lp_glsl::FixedPointFormat>,
    return_type: ReturnType,
    execute_fn: impl FnOnce(
        &crate::execution::EmulatorBackend,
        &crate::execution::backend::CompiledCode,
        Option<lp_glsl::FixedPointFormat>,
    ) -> Result<T>,
) -> Result<T> {
    use crate::execution::binary::compile_to_binary;
    use crate::execution::{CompiledCode, EmulatorBackend, EmulatorType};

    // Compile GLSL to binary (bootstrap + test function)
    let binary = compile_to_binary(glsl_source, fixed_point_format, return_type)?;

    // Create compiled code
    let compiled_code = CompiledCode::EmulatorBinary {
        binary,
        return_type,
    };

    // Execute using emulator backend
    let backend = EmulatorBackend::new(EmulatorType::Riscv32);
    execute_fn(&backend, &compiled_code, fixed_point_format)
}

/// Execute GLSL code in riscv32 emulator and return float result
fn execute_riscv32_float(
    glsl_source: &str,
    fixed_point_format: Option<lp_glsl::FixedPointFormat>,
) -> Result<f32> {
    use crate::execution::ExecutionBackend;
    execute_riscv32(
        glsl_source,
        fixed_point_format,
        ReturnType::Float,
        |backend, code, fmt| backend.execute_float(code, fmt),
    )
}
