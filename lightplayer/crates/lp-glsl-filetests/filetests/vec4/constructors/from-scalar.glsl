// test run
// target riscv32.fixed32

// ============================================================================
// Vector constructor from single scalar: vec4(float) - all components same value
// ============================================================================

float test_vec4_from_scalar() {
    vec4 v = vec4(5.0);
    // All components should be 5.0
    return v.x + v.y + v.z + v.w;
    // Should be 5.0 + 5.0 + 5.0 + 5.0 = 20.0
}

// run: test_vec4_from_scalar() ~= 20.0

float test_vec4_from_scalar_zero() {
    vec4 v = vec4(0.0);
    // All components should be 0.0
    return v.x + v.y + v.z + v.w;
    // Should be 0.0 + 0.0 + 0.0 + 0.0 = 0.0
}

// run: test_vec4_from_scalar_zero() ~= 0.0

float test_vec4_from_scalar_negative() {
    vec4 v = vec4(-3.5);
    // All components should be -3.5
    return v.x + v.y + v.z + v.w;
    // Should be -3.5 + -3.5 + -3.5 + -3.5 = -14.0
}

// run: test_vec4_from_scalar_negative() ~= -14.0

float test_vec4_from_scalar_verify_components() {
    vec4 v = vec4(7.0);
    // Verify each component individually
    float sum = 0.0;
    if (v.x == 7.0) sum = sum + 1.0;
    if (v.y == 7.0) sum = sum + 1.0;
    if (v.z == 7.0) sum = sum + 1.0;
    if (v.w == 7.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components match)
}

// run: test_vec4_from_scalar_verify_components() ~= 4.0

