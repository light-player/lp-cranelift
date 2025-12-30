// test run
// target riscv32.fixed32

// ============================================================================
// Phase 3: Inverse Trig Functions (asin, acos, atan, atan2)
// Acceptance test: Verify inverse trig functions work
// ============================================================================

float test_asin_basic() {
    // Test that asin() works
    return asin(1.0);
}

// run: test_asin_basic() ~= 1.5707963267948966 // π/2

float test_acos_basic() {
    // Test that acos() works
    return acos(0.0);
}

// run: test_acos_basic() ~= 1.5707963267948966 // π/2

float test_atan_basic() {
    // Test that atan() works (single arg)
    return atan(1.0);
}

// run: test_atan_basic() ~= 0.7853981633974483 // π/4

float test_atan2_basic() {
    // Test that atan2() works (two args)
    return atan(1.0, 0.0); // atan2(y, x)
}

// run: test_atan2_basic() ~= 1.5707963267948966 // π/2

vec2 test_inverse_trig_vec2() {
    // Test component-wise operation
    return asin(vec2(0.0, 1.0));
}

// run: test_inverse_trig_vec2() ~= vec2(0.0, 1.5707963267948966)



