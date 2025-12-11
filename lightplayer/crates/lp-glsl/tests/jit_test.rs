//! JIT execution tests using the new API

use lp_glsl::{DecimalFormat, GlslOptions, RunMode, glsl_jit};

#[test]
fn test_jit_int_literal() {
    let source = r#"
        int main() {
            return 42;
        }
    "#;

    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };

    let mut executable = glsl_jit(source, options).expect("Compilation failed");
    let result = executable.call_i32("main", &[]).expect("Execution failed");
    assert_eq!(result, 42);
}

#[test]
fn test_jit_int_addition() {
    let source = r#"
        int main() {
            int a = 10;
            int b = 20;
            return a + b;
        }
    "#;

    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };

    let mut executable = glsl_jit(source, options).expect("Compilation failed");
    let result = executable.call_i32("main", &[]).expect("Execution failed");
    assert_eq!(result, 30);
}

#[test]
fn test_jit_float_literal() {
    let source = r#"
        float main() {
            return 3.14;
        }
    "#;

    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };

    let mut executable = glsl_jit(source, options).expect("Compilation failed");
    let result = executable.call_f32("main", &[]).expect("Execution failed");
    assert!((result - 3.14).abs() < 0.01);
}

#[test]
fn test_jit_bool_literal() {
    let source = r#"
        bool main() {
            return true;
        }
    "#;

    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };

    let mut executable = glsl_jit(source, options).expect("Compilation failed");
    let result = executable.call_bool("main", &[]).expect("Execution failed");
    assert_eq!(result, true);
}
