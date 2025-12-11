//! Emulator execution tests with fixed-point using the new API

#[cfg(feature = "emulator")]
use lp_glsl::{GlslOptions, glsl_emu_riscv32};

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
#[test]
fn test_emu_int_literal() {
    let source = r#"
        int main() {
            return 42;
        }
    "#;

    let options = GlslOptions::emu_riscv32_imac();

    let mut executable = glsl_emu_riscv32(source, options).expect("Compilation failed");
    let result = executable.call_i32("main", &[]).expect("Execution failed");
    assert_eq!(result, 42);
}

#[cfg(feature = "emulator")]
#[test]
fn test_emu_int_addition() {
    let source = r#"
        int main() {
            int a = 10;
            int b = 20;
            return a + b;
        }
    "#;

    let options = GlslOptions::emu_riscv32_imac();

    let mut executable = glsl_emu_riscv32(source, options).expect("Compilation failed");
    let result = executable.call_i32("main", &[]).expect("Execution failed");
    assert_eq!(result, 30);
}

#[cfg(feature = "emulator")]
#[test]
fn test_emu_float_constant_fixed32() {
    let source = r#"
        float main() {
            return 3.14159;
        }
    "#;

    let options = GlslOptions::emu_riscv32_imac();

    let mut executable = glsl_emu_riscv32(source, options).expect("Compilation failed");
    let result = executable.call_f32("main", &[]).expect("Execution failed");

    // The emulator returns fixed-point as f32, so we need to check the fixed-point value
    let expected_fixed = float_to_fixed32(3.14159);
    let result_fixed = float_to_fixed32(result);

    // Allow some tolerance for fixed-point conversion
    assert!(
        (result_fixed - expected_fixed).abs() < 10,
        "Expected fixed-point value around {}, got {}",
        expected_fixed,
        result_fixed
    );

    // Verify it's approximately correct as float
    assert!((result - 3.14159).abs() < 0.0001);
}

#[cfg(feature = "emulator")]
#[test]
fn test_emu_float_addition_fixed32() {
    let source = r#"
        float main() {
            float a = 2.5;
            float b = 1.25;
            return a + b;
        }
    "#;

    let options = GlslOptions::emu_riscv32_imac();

    let mut executable = glsl_emu_riscv32(source, options).expect("Compilation failed");
    let result = executable.call_f32("main", &[]).expect("Execution failed");

    let expected = 3.75;
    let result_float = fixed32_to_float(float_to_fixed32(result));
    assert!(
        (result_float - expected).abs() < 0.0001,
        "Expected ~{}, got {}",
        expected,
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_emu_float_multiplication_fixed32() {
    let source = r#"
        float main() {
            float a = 2.0;
            float b = 3.5;
            return a * b;
        }
    "#;

    let options = GlslOptions::emu_riscv32_imac();

    let mut executable = glsl_emu_riscv32(source, options).expect("Compilation failed");
    let result = executable.call_f32("main", &[]).expect("Execution failed");

    let expected = 7.0;
    let result_float = fixed32_to_float(float_to_fixed32(result));
    assert!(
        (result_float - expected).abs() < 0.001,
        "Expected ~{}, got {}",
        expected,
        result_float
    );
}

#[cfg(feature = "emulator")]
#[test]
fn test_emu_user_fn_fixed32() {
    let source = r#"
        float main() {
            float a = 2.0;
            float b = 3.5;
            return multiply(a, b);
        }

        float multiply(float a, float b) {
            return a * b;
        }
    "#;

    let options = GlslOptions::emu_riscv32_imac();

    let mut executable = glsl_emu_riscv32(source, options).expect("Compilation failed");
    let result = executable.call_f32("main", &[]).expect("Execution failed");

    let expected = 7.0;
    let result_float = fixed32_to_float(float_to_fixed32(result));
    assert!(
        (result_float - expected).abs() < 0.001,
        "Expected ~{}, got {}",
        expected,
        result_float
    );
}
