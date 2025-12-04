use lp_glsl::Compiler;

/// Compile and execute GLSL that returns i32
pub fn run_int_test(source: &str) -> i32 {
    let mut compiler = Compiler::new();
    let func = compiler.compile_int(source).expect("Compilation failed");
    func()
}

/// Compile and execute GLSL that returns i8 (bool)
pub fn run_bool_test(source: &str) -> i8 {
    let mut compiler = Compiler::new();
    let func = compiler.compile_bool(source).expect("Compilation failed");
    func()
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

/// Assert that GLSL code produces expected boolean result (as i8)
#[macro_export]
macro_rules! assert_bool_result {
    ($source:expr, $expected:expr) => {
        let result = common::run_bool_test($source);
        let expected_val: i8 = if $expected { 1 } else { 0 };
        assert_eq!(
            result, expected_val,
            "Expected {}, got {} for:\n{}",
            $expected, result, $source
        );
    };
}

