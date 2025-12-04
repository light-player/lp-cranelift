mod common;

use lp_glsl::{Compiler, FixedPointFormat};

/// Compile and execute GLSL with fixed-point transformation that returns i32
fn run_fixed16_test(source: &str) -> i32 {
    let mut compiler = Compiler::new();
    compiler.set_fixed_point_format(Some(FixedPointFormat::Fixed16x16));
    let func = compiler.compile_int(source).expect("Compilation failed");
    func()
}

/// Convert float to 16.16 fixed-point for comparison
fn float_to_fixed16(f: f32) -> i32 {
    let clamped = f.clamp(-32768.0, 32767.9999847412109375);
    let scaled = clamped * 65536.0;
    if scaled >= 0.0 {
        (scaled + 0.5) as i32
    } else {
        (scaled - 0.5) as i32
    }
}

/// Convert fixed-point back to float
fn fixed16_to_float(fixed: i32) -> f32 {
    fixed as f32 / 65536.0
}

#[test]
fn test_float_constant_fixed16() {
    let shader = r#"
        float main() {
            return 3.14159;
        }
    "#;
    let result = run_fixed16_test(shader);
    let expected = float_to_fixed16(3.14159);
    
    assert_eq!(result, expected, "Float constant conversion mismatch");
    
    // Verify it's approximately correct
    let result_float = fixed16_to_float(result);
    assert!((result_float - 3.14159).abs() < 0.0001);
}

#[test]
fn test_float_addition_fixed16() {
    let shader = r#"
        float main() {
            float a = 2.5;
            float b = 1.25;
            return a + b;
        }
    "#;
    let result = run_fixed16_test(shader);
    let expected = float_to_fixed16(3.75);
    
    assert_eq!(result, expected, "Float addition mismatch");
    
    let result_float = fixed16_to_float(result);
    assert!((result_float - 3.75).abs() < 0.0001);
}

#[test]
fn test_float_subtraction_fixed16() {
    let shader = r#"
        float main() {
            float a = 5.5;
            float b = 2.25;
            return a - b;
        }
    "#;
    let result = run_fixed16_test(shader);
    let expected = float_to_fixed16(3.25);
    
    assert_eq!(result, expected, "Float subtraction mismatch");
    
    let result_float = fixed16_to_float(result);
    assert!((result_float - 3.25).abs() < 0.0001);
}

#[test]
fn test_float_multiplication_fixed16() {
    let shader = r#"
        float main() {
            float a = 2.0;
            float b = 3.5;
            return a * b;
        }
    "#;
    let result = run_fixed16_test(shader);
    let expected = float_to_fixed16(7.0);
    
    // Allow some tolerance for fixed-point multiplication
    let result_float = fixed16_to_float(result);
    assert!((result_float - 7.0).abs() < 0.001, 
        "Expected ~7.0, got {}", result_float);
}

#[test]
fn test_float_division_fixed16() {
    let shader = r#"
        float main() {
            float a = 10.0;
            float b = 4.0;
            return a / b;
        }
    "#;
    let result = run_fixed16_test(shader);
    
    // Allow some tolerance for fixed-point division
    let result_float = fixed16_to_float(result);
    assert!((result_float - 2.5).abs() < 0.001, 
        "Expected ~2.5, got {}", result_float);
}

#[test]
fn test_float_complex_expr_fixed16() {
    let shader = r#"
        float main() {
            float a = 2.0;
            float b = 3.0;
            float c = 4.0;
            return (a + b) * c;
        }
    "#;
    let result = run_fixed16_test(shader);
    
    // (2.0 + 3.0) * 4.0 = 5.0 * 4.0 = 20.0
    let result_float = fixed16_to_float(result);
    assert!((result_float - 20.0).abs() < 0.01, 
        "Expected ~20.0, got {}", result_float);
}

#[test]
fn test_negative_numbers_fixed16() {
    let shader = r#"
        float main() {
            float a = -5.5;
            float b = 2.25;
            return a + b;
        }
    "#;
    let result = run_fixed16_test(shader);
    
    // -5.5 + 2.25 = -3.25
    let result_float = fixed16_to_float(result);
    assert!((result_float - (-3.25)).abs() < 0.001, 
        "Expected ~-3.25, got {}", result_float);
}

#[test]
fn test_fractional_precision_fixed16() {
    let shader = r#"
        float main() {
            float a = 0.0625;
            float b = 0.03125;
            return a + b;
        }
    "#;
    let result = run_fixed16_test(shader);
    
    // 0.0625 + 0.03125 = 0.09375
    let result_float = fixed16_to_float(result);
    assert!((result_float - 0.09375).abs() < 0.0001, 
        "Expected ~0.09375, got {}", result_float);
}

