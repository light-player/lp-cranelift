//! Test JIT execution with result verification
//! Pattern: cranelift/filetests/src/test_run.rs

use anyhow::Result;

pub fn run_test(_full_source: &str, glsl_source: &str, fixed_point_format: Option<lp_glsl::FixedPointFormat>) -> Result<()> {
    // Parse run directives: // run: <expected_result>
    let run_directives = parse_run_directives(_full_source)?;
    
    if run_directives.is_empty() {
        anyhow::bail!("No 'run' directives found");
    }
    
    // Compile and execute
    for directive in run_directives {
        match directive.expected_type {
            ExpectedType::Int(expected) => {
                let mut compiler = lp_glsl::Compiler::new();
                //compiler.set_fixed_point_format(_fixed_point_format);
                let func = compiler.compile_int(glsl_source)
                    .map_err(|e| anyhow::anyhow!("Failed to compile for run test: {}", e))?;
                let result = func();
                
                if result != expected {
                    anyhow::bail!(
                        "Run test failed: expected {}, got {}",
                        expected,
                        result
                    );
                }
            }
            ExpectedType::Bool(expected) => {
                let mut compiler = lp_glsl::Compiler::new();
                compiler.set_fixed_point_format(fixed_point_format);
                let func = compiler.compile_bool(glsl_source)
                    .map_err(|e| anyhow::anyhow!("Failed to compile for run test: {}", e))?;
                let result = func();
                let expected_val = if expected { 1 } else { 0 };
                
                if result != expected_val {
                    anyhow::bail!(
                        "Run test failed: expected {}, got {}",
                        expected,
                        result != 0
                    );
                }
            }
            ExpectedType::FloatApprox { expected, tolerance } => {
                // Skip 32.32 runtime tests for now - they require i64 return type
                if let Some(lp_glsl::FixedPointFormat::Fixed32x32) = fixed_point_format {
                    // TODO: Add compile_i64 method to support 32.32 runtime tests
                    return Ok(());
                }
                
                let mut compiler = lp_glsl::Compiler::new();
                compiler.set_fixed_point_format(fixed_point_format);
                let func = compiler.compile_int(glsl_source)
                    .map_err(|e| anyhow::anyhow!("Failed to compile for run test: {}", e))?;
                let result_fixed = func();
                
                // Convert fixed-point result back to float
                let result_float = if let Some(format) = fixed_point_format {
                    match format {
                        lp_glsl::FixedPointFormat::Fixed16x16 => result_fixed as f32 / 65536.0,
                        lp_glsl::FixedPointFormat::Fixed32x32 => {
                            // This path won't be reached due to early return above
                            unreachable!()
                        }
                    }
                } else {
                    // No fixed-point, interpret as raw float bits
                    f32::from_bits(result_fixed as u32)
                };
                
                let diff = (result_float - expected).abs();
                if diff > tolerance {
                    anyhow::bail!(
                        "Run test failed: expected {} (tolerance {}), got {} (diff: {})",
                        expected,
                        tolerance,
                        result_float,
                        diff
                    );
                }
            }
        }
    }
    
    Ok(())
}

struct RunDirective {
    expected_type: ExpectedType,
}

enum ExpectedType {
    Int(i32),
    Bool(bool),
    FloatApprox { expected: f32, tolerance: f32 },
}

fn parse_run_directives(source: &str) -> Result<Vec<RunDirective>> {
    let mut directives = Vec::new();
    
    for line in source.lines() {
        let trimmed = line.trim();
        // Parse // run: directives
        if let Some(comment) = trimmed.strip_prefix("//") {
            if let Some(run_spec) = comment.trim().strip_prefix("run:") {
                let spec = run_spec.trim();
            
                // Parse "~= <value> (tolerance: <tol>)" for approximate float comparison
                if let Some(approx_str) = spec.strip_prefix("~=").map(str::trim) {
                    // Parse "value (tolerance: tolerance)"
                    if let Some((value_str, tolerance_part)) = approx_str.split_once("(tolerance:") {
                        let value = value_str.trim().parse::<f32>()
                            .map_err(|_| anyhow::anyhow!("Failed to parse float value: {}", value_str))?;
                        let tolerance_str = tolerance_part.trim().trim_end_matches(')').trim();
                        let tolerance = tolerance_str.parse::<f32>()
                            .map_err(|_| anyhow::anyhow!("Failed to parse tolerance: {}", tolerance_str))?;
                        directives.push(RunDirective {
                            expected_type: ExpectedType::FloatApprox { expected: value, tolerance },
                        });
                    }
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

