use lp_glsl::{DecimalFormat, GlslOptions, RunMode, glsl_jit};

/// Compile and execute GLSL that returns i32 using JIT
#[allow(dead_code)] // Used by assert_int_result! macro
pub fn run_int_test(source: &str) -> i32 {
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    let mut executable = glsl_jit(source, options).expect("Compilation failed");
    executable.call_i32("main", &[]).expect("Execution failed")
}

/// Compile and execute GLSL that returns bool using JIT
#[allow(dead_code)] // Used by assert_bool_result! macro
pub fn run_bool_test(source: &str) -> bool {
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    let mut executable = glsl_jit(source, options).expect("Compilation failed");
    executable.call_bool("main", &[]).expect("Execution failed")
}

/// Assert that GLSL code produces expected integer result
#[macro_export]
macro_rules! assert_int_result {
    ($source:expr, $expected:expr) => {
        let result = common::run_int_test($source);
        assert_eq!(
            result, $expected,
            "Expected {}, got {} for:\n{}",
            $expected, result, $source
        );
    };
}

/// Assert that GLSL code produces expected boolean result
#[macro_export]
macro_rules! assert_bool_result {
    ($source:expr, $expected:expr) => {
        let result = common::run_bool_test($source);
        assert_eq!(
            result, $expected,
            "Expected {}, got {} for:\n{}",
            $expected, result, $source
        );
    };
}
