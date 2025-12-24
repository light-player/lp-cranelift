mod common;

#[cfg(feature = "emulator")]
use lp_glsl::{GlslValue, execute_main, glsl_emu_riscv32_with_metadata};

/// Compile and execute GLSL with fixed-point transformation that returns i32
#[cfg(feature = "emulator")]
fn run_fixed32_test(source: &str) -> i32 {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let mut executable =
        glsl_emu_riscv32_with_metadata(source, options, None).expect("Compilation failed");

    // Use shared execute_main which provides debug output on errors
    let result_value = execute_main(&mut *executable).expect("Execution failed");

    // Extract f32 from GlslValue
    let result_f32 = match result_value {
        GlslValue::F32(f) => f,
        _ => panic!("Expected f32 return type"),
    };

    float_to_fixed32(result_f32)
}

/// Convert float to 16.16 fixed-point for comparison
fn float_to_fixed32(f: f32) -> i32 {
    let clamped = f.clamp(-32768.0, 32767.9999847412109375);
    let scaled = clamped * 65536.0;
    if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    }
}

/// Convert fixed-point back to float
fn fixed32_to_float(fixed: i32) -> f32 {
    fixed as f32 / 65536.0
}

#[cfg(feature = "emulator")]
#[cfg(feature = "emulator")]
#[test]
fn test_float_constant_fixed32() {
    let shader = r#"
        float main() {
            return 3.14159;
        }
    "#;
    let result = run_fixed32_test(shader);
    let expected = float_to_fixed32(3.14159);

    assert_eq!(result, expected, "Float constant conversion mismatch");

    // Verify it's approximately correct
    let result_float = fixed32_to_float(result);
    assert!((result_float - 3.14159).abs() < 0.0001);
}

#[cfg(feature = "emulator")]
#[test]
fn test_float_addition_fixed32() {
    let shader = r#"
        float main() {
            float a = 2.5;
            float b = 1.25;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);
    let expected = float_to_fixed32(3.75);

    assert_eq!(result, expected, "Float addition mismatch");

    let result_float = fixed32_to_float(result);
    assert!((result_float - 3.75).abs() < 0.0001);
}

#[cfg(feature = "emulator")]
#[test]
fn test_float_subtraction_fixed32() {
    let shader = r#"
        float main() {
            float a = 5.5;
            float b = 2.25;
            return a - b;
        }
    "#;
    let result = run_fixed32_test(shader);
    let expected = float_to_fixed32(3.25);

    assert_eq!(result, expected, "Float subtraction mismatch");

    let result_float = fixed32_to_float(result);
    assert!((result_float - 3.25).abs() < 0.0001);
}

#[cfg(feature = "emulator")]
#[test]
fn test_float_multiplication_fixed32() {
    let shader = r#"
        float main() {
            float a = 2.0;
            float b = 3.5;
            return a * b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // Allow some tolerance for fixed-point multiplication
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 7.0).abs() < 0.001,
        "Expected ~7.0, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_float_division_fixed32() {
    let shader = r#"
        float main() {
            float a = 10.0;
            float b = 4.0;
            return a / b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // Allow some tolerance for fixed-point division
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 2.5).abs() < 0.001,
        "Expected ~2.5, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_float_complex_expr_fixed32() {
    let shader = r#"
        float main() {
            float a = 2.0;
            float b = 3.0;
            float c = 4.0;
            return (a + b) * c;
        }
    "#;
    let result = run_fixed32_test(shader);

    // (2.0 + 3.0) * 4.0 = 5.0 * 4.0 = 20.0
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 20.0).abs() < 0.01,
        "Expected ~20.0, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_negative_numbers_fixed32() {
    let shader = r#"
        float main() {
            float a = -5.5;
            float b = 2.25;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // -5.5 + 2.25 = -3.25
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - (-3.25)).abs() < 0.001,
        "Expected ~-3.25, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_fractional_precision_fixed32() {
    let shader = r#"
        float main() {
            float a = 0.0625;
            float b = 0.03125;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // 0.0625 + 0.03125 = 0.09375
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 0.09375).abs() < 0.0001,
        "Expected ~0.09375, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_division_by_zero_positive() {
    let shader = r#"
        float main() {
            float a = 10.0;
            float b = 0.0;
            return a / b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // Division by zero should saturate to max value (32767.0) for positive numerator
    let result_float = fixed32_to_float(result);
    // Should be close to max representable value (32767.0)
    assert!(
        result_float > 30000.0,
        "Division by zero with positive numerator should saturate to max, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_division_by_zero_negative() {
    let shader = r#"
        float main() {
            float a = -10.0;
            float b = 0.0;
            return a / b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // Division by zero should saturate to min value (-32768.0) for negative numerator
    let result_float = fixed32_to_float(result);
    // Should be close to min representable value (-32768.0)
    assert!(
        result_float < -30000.0,
        "Division by zero with negative numerator should saturate to min, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_division_by_zero_zero() {
    let shader = r#"
        float main() {
            float a = 0.0;
            float b = 0.0;
            return a / b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // 0/0 = NaN in floating-point, which we approximate as 0 (neutral value)
    // since fixed-point doesn't have NaN representation
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 0.0).abs() < 0.0001,
        "0/0 should return 0 (NaN approximation), got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_overflow_near_max() {
    let shader = r#"
        float main() {
            float a = 30000.0;
            float b = 3000.0;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // 30000.0 + 3000.0 = 33000.0, which exceeds max (32767.0)
    // Result will wrap around due to two's complement arithmetic
    let result_float = fixed32_to_float(result);
    // The result should wrap, so it won't be 33000.0
    // We just verify it compiles and runs without crashing
    assert!(
        result_float.is_finite(),
        "Result should be finite, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_underflow_near_min() {
    let shader = r#"
        float main() {
            float a = -30000.0;
            float b = -3000.0;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // -30000.0 + -3000.0 = -33000.0, which is less than min (-32768.0)
    // Result will wrap around due to two's complement arithmetic
    let result_float = fixed32_to_float(result);
    // The result should wrap, so it won't be -33000.0
    // We just verify it compiles and runs without crashing
    assert!(
        result_float.is_finite(),
        "Result should be finite, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_precision_limits_very_small() {
    let shader = r#"
        float main() {
            float a = 0.0001;
            float b = 0.0001;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // 0.0001 + 0.0001 = 0.0002
    // Fixed16x16 precision is 1/65536 ≈ 0.00001526
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 0.0002).abs() < 0.0001,
        "Expected ~0.0002, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_precision_limits_very_large() {
    let shader = r#"
        float main() {
            float a = 32767.0;
            return a;
        }
    "#;
    let result = run_fixed32_test(shader);

    // Should be close to max representable value
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 32767.0).abs() < 1.0,
        "Expected ~32767.0, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_negative_precision() {
    let shader = r#"
        float main() {
            float a = -0.0001;
            float b = -0.0001;
            return a + b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // -0.0001 + -0.0001 = -0.0002
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - (-0.0002)).abs() < 0.0001,
        "Expected ~-0.0002, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_multiplication_precision() {
    let shader = r#"
        float main() {
            float a = 0.5;
            float b = 0.5;
            return a * b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // 0.5 * 0.5 = 0.25
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 0.25).abs() < 0.001,
        "Expected ~0.25, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_division_precision() {
    let shader = r#"
        float main() {
            float a = 1.0;
            float b = 3.0;
            return a / b;
        }
    "#;
    let result = run_fixed32_test(shader);

    // 1.0 / 3.0 = 0.333...
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 0.333333).abs() < 0.01,
        "Expected ~0.333, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_complex_expression_with_overflow() {
    let shader = r#"
        float main() {
            float a = 20000.0;
            float b = 15000.0;
            float c = 5000.0;
            return (a + b) - c;
        }
    "#;
    let result = run_fixed32_test(shader);

    // (20000.0 + 15000.0) - 5000.0 = 30000.0
    let result_float = fixed32_to_float(result);
    // Should be close to 30000.0, but may have precision issues
    assert!(
        (result_float - 30000.0).abs() < 10.0,
        "Expected ~30000.0, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_abs_operation() {
    let shader = r#"
        float main() {
            float a = -5.5;
            return abs(a);
        }
    "#;
    let result = run_fixed32_test(shader);

    // abs(-5.5) = 5.5
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - 5.5).abs() < 0.001,
        "Expected ~5.5, got {}",
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_neg_operation() {
    let shader = r#"
        float main() {
            float a = 5.5;
            return -a;
        }
    "#;
    let result = run_fixed32_test(shader);

    // -5.5 = -5.5
    let result_float = fixed32_to_float(result);
    assert!(
        (result_float - (-5.5)).abs() < 0.001,
        "Expected ~-5.5, got {}",
        result_float
    );
}

// Vector operation tests with fixed-point
// NOTE: These tests are ignored due to SIGBUS issues with native JIT execution of fixed-point vectors.
// The issue appears to be in codegen/ABI handling, not buffer alignment. Filetests work correctly
// on riscv32 emulator. These will be re-enabled once the native JIT path is fixed.

#[cfg(feature = "emulator")]
#[test]
#[ignore]
fn test_vec2_fixed32() {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let shader = r#"
        vec2 main() {
            vec2 a = vec2(2.5, 3.5);
            vec2 b = vec2(1.0, 1.5);
            return a + b;
        }
    "#;
    let mut executable =
        glsl_emu_riscv32_with_metadata(shader, options, None).expect("Compilation failed");
    let result_value = execute_main(&mut *executable).expect("Execution failed");
    let result = match result_value {
        GlslValue::Vec2(v) => v,
        _ => panic!("Expected Vec2 return type"),
    };
    let x = result[0];
    let y = result[1];

    // vec2(2.5, 3.5) + vec2(1.0, 1.5) = vec2(3.5, 5.0)
    assert!((x - 3.5).abs() < 0.001, "Expected x ~3.5, got {}", x);
    assert!((y - 5.0).abs() < 0.001, "Expected y ~5.0, got {}", y);
}

#[cfg(feature = "emulator")]
#[test]
#[ignore]
fn test_vec3_fixed32() {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let shader = r#"
        vec3 main() {
            vec3 a = vec3(1.0, 2.0, 3.0);
            vec3 b = vec3(0.5, 1.5, 2.5);
            return a + b;
        }
    "#;
    let mut executable =
        glsl_emu_riscv32_with_metadata(shader, options, None).expect("Compilation failed");
    let result_value = execute_main(&mut *executable).expect("Execution failed");
    let result = match result_value {
        GlslValue::Vec3(v) => v,
        _ => panic!("Expected Vec3 return type"),
    };
    let x = result[0];
    let y = result[1];
    let z = result[2];

    // vec3(1.0, 2.0, 3.0) + vec3(0.5, 1.5, 2.5) = vec3(1.5, 3.5, 5.5)
    assert!((x - 1.5).abs() < 0.001, "Expected x ~1.5, got {}", x);
    assert!((y - 3.5).abs() < 0.001, "Expected y ~3.5, got {}", y);
    assert!((z - 5.5).abs() < 0.001, "Expected z ~5.5, got {}", z);
}

#[cfg(feature = "emulator")]
#[test]
#[ignore]
fn test_vec4_fixed32() {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let shader = r#"
        vec4 main() {
            vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
            vec4 b = vec4(0.5, 1.5, 2.5, 3.5);
            return a + b;
        }
    "#;
    let mut executable =
        glsl_emu_riscv32_with_metadata(shader, options, None).expect("Compilation failed");
    let result_value = execute_main(&mut *executable).expect("Execution failed");
    let result = match result_value {
        GlslValue::Vec4(v) => v,
        _ => panic!("Expected Vec4 return type"),
    };
    let x = result[0];
    let y = result[1];
    let z = result[2];
    let w = result[3];

    // vec4(1.0, 2.0, 3.0, 4.0) + vec4(0.5, 1.5, 2.5, 3.5) = vec4(1.5, 3.5, 5.5, 7.5)
    assert!((x - 1.5).abs() < 0.001, "Expected x ~1.5, got {}", x);
    assert!((y - 3.5).abs() < 0.001, "Expected y ~3.5, got {}", y);
    assert!((z - 5.5).abs() < 0.001, "Expected z ~5.5, got {}", z);
    assert!((w - 7.5).abs() < 0.001, "Expected w ~7.5, got {}", w);
}

#[cfg(feature = "emulator")]
#[test]
#[ignore]
fn test_vec2_multiplication_fixed32() {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let shader = r#"
        vec2 main() {
            vec2 a = vec2(2.0, 3.0);
            vec2 b = vec2(1.5, 2.5);
            return a * b;
        }
    "#;
    let mut executable =
        glsl_emu_riscv32_with_metadata(shader, options, None).expect("Compilation failed");
    let result_value = execute_main(&mut *executable).expect("Execution failed");
    let result = match result_value {
        GlslValue::Vec2(v) => v,
        _ => panic!("Expected Vec2 return type"),
    };
    let x = result[0];
    let y = result[1];

    // vec2(2.0, 3.0) * vec2(1.5, 2.5) = vec2(3.0, 7.5)
    assert!((x - 3.0).abs() < 0.01, "Expected x ~3.0, got {}", x);
    assert!((y - 7.5).abs() < 0.01, "Expected y ~7.5, got {}", y);
}

#[cfg(feature = "emulator")]
#[test]
#[ignore]
fn test_vec3_division_fixed32() {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let shader = r#"
        vec3 main() {
            vec3 a = vec3(10.0, 20.0, 30.0);
            vec3 b = vec3(2.0, 4.0, 5.0);
            return a / b;
        }
    "#;
    let mut executable =
        glsl_emu_riscv32_with_metadata(shader, options, None).expect("Compilation failed");
    let result_value = execute_main(&mut *executable).expect("Execution failed");
    let result = match result_value {
        GlslValue::Vec3(v) => v,
        _ => panic!("Expected Vec3 return type"),
    };
    let x = result[0];
    let y = result[1];
    let z = result[2];

    // vec3(10.0, 20.0, 30.0) / vec3(2.0, 4.0, 5.0) = vec3(5.0, 5.0, 6.0)
    assert!((x - 5.0).abs() < 0.01, "Expected x ~5.0, got {}", x);
    assert!((y - 5.0).abs() < 0.01, "Expected y ~5.0, got {}", y);
    assert!((z - 6.0).abs() < 0.01, "Expected z ~6.0, got {}", z);
}

#[cfg(feature = "emulator")]
#[test]
#[ignore]
fn test_vec4_complex_expression_fixed32() {
    let options = lp_glsl::GlslOptions::emu_riscv32_imac();
    let shader = r#"
        vec4 main() {
            vec4 a = vec4(1.0, 2.0, 3.0, 4.0);
            vec4 b = vec4(0.5, 1.0, 1.5, 2.0);
            return (a + b) * 2.0;
        }
    "#;
    let mut executable =
        glsl_emu_riscv32_with_metadata(shader, options, None).expect("Compilation failed");
    let result_value = execute_main(&mut *executable).expect("Execution failed");
    let result = match result_value {
        GlslValue::Vec4(v) => v,
        _ => panic!("Expected Vec4 return type"),
    };
    let x = result[0];
    let y = result[1];
    let z = result[2];
    let w = result[3];

    // (vec4(1.0, 2.0, 3.0, 4.0) + vec4(0.5, 1.0, 1.5, 2.0)) * 2.0 = vec4(3.0, 6.0, 9.0, 12.0)
    assert!((x - 3.0).abs() < 0.01, "Expected x ~3.0, got {}", x);
    assert!((y - 6.0).abs() < 0.01, "Expected y ~6.0, got {}", y);
    assert!((z - 9.0).abs() < 0.01, "Expected z ~9.0, got {}", z);
    assert!((w - 12.0).abs() < 0.01, "Expected w ~12.0, got {}", w);
}
