// test run
// target riscv32.fixed32

// ============================================================================
// Phase 5: Exponential Functions (exp, log, exp2, log2, pow)
// Acceptance test: Verify exponential functions work
// ============================================================================

float test_exp_basic() {
    // Test that exp() works
    return exp(0.0);
}

// run: test_exp_basic() ~= 1.0

float test_log_basic() {
    // Test that log() works
    return log(1.0);
}

// run: test_log_basic() ~= 0.0

float test_exp2_basic() {
    // Test that exp2() works
    return exp2(0.0);
}

// run: test_exp2_basic() ~= 1.0

float test_log2_basic() {
    // Test that log2() works
    return log2(1.0);
}

// run: test_log2_basic() ~= 0.0

float test_pow_basic() {
    // Test that pow() works
    return pow(2.0, 2.0);
}

// run: test_pow_basic() ~= 4.0

vec2 test_exponential_vec2() {
    // Test component-wise operation
    return exp(vec2(0.0, 1.0));
}

// run: test_exponential_vec2() ~= vec2(1.0, 2.718281828459045)


