// test run
// target riscv32.fixed32

// ============================================================================
// Phase 4: Hyperbolic Trig Functions (sinh, cosh, tanh, asinh, acosh, atanh)
// Acceptance test: Verify hyperbolic trig functions work
// ============================================================================

float test_sinh_basic() {
    // Test that sinh() works
    return sinh(0.0);
}

// run: test_sinh_basic() ~= 0.0

float test_cosh_basic() {
    // Test that cosh() works
    return cosh(0.0);
}

// run: test_cosh_basic() ~= 1.0

float test_tanh_basic() {
    // Test that tanh() works
    return tanh(0.0);
}

// run: test_tanh_basic() ~= 0.0

float test_asinh_basic() {
    // Test that asinh() works
    return asinh(0.0);
}

// run: test_asinh_basic() ~= 0.0

float test_acosh_basic() {
    // Test that acosh() works
    return acosh(1.0);
}

// run: test_acosh_basic() ~= 0.0

float test_atanh_basic() {
    // Test that atanh() works
    return atanh(0.0);
}

// run: test_atanh_basic() ~= 0.0

vec2 test_hyperbolic_trig_vec2() {
    // Test component-wise operation
    return sinh(vec2(0.0, 1.0));
}

// run: test_hyperbolic_trig_vec2() ~= vec2(0.0, 1.1752011936438014)


