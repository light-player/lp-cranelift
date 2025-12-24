// test run
// target riscv32.fixed32

// ============================================================================
// Vector negation: -vec4 (component-wise)
// ============================================================================

float test_vec4_negation() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = -v;
    // Component-wise negation
    return result.x + result.y + result.z + result.w;
    // Should be -1.0 + -2.0 + -3.0 + -4.0 = -10.0
}

// run: test_vec4_negation() ~= -10.0

float test_vec4_negation_verify_components() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = -v;
    // Verify each component
    float sum = 0.0;
    if (result.x == -10.0) sum = sum + 1.0;
    if (result.y == -20.0) sum = sum + 1.0;
    if (result.z == -30.0) sum = sum + 1.0;
    if (result.w == -40.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components correct)
}

// run: test_vec4_negation_verify_components() ~= 4.0

float test_vec4_negation_negative() {
    vec4 v = vec4(-1.0, -2.0, -3.0, -4.0);
    vec4 result = -v;
    // Negation of negative vector (should be positive)
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_negation_negative() ~= 10.0

float test_vec4_negation_mixed() {
    vec4 v = vec4(1.0, -2.0, 3.0, -4.0);
    vec4 result = -v;
    // Negation of mixed positive/negative
    return result.x + result.y + result.z + result.w;
    // Should be -1.0 + 2.0 + -3.0 + 4.0 = 2.0
}

// run: test_vec4_negation_mixed() ~= 2.0

float test_vec4_negation_zero() {
    vec4 v = vec4(0.0, 0.0, 0.0, 0.0);
    vec4 result = -v;
    // Negation of zero vector
    return result.x + result.y + result.z + result.w;
    // Should be 0.0 + 0.0 + 0.0 + 0.0 = 0.0
}

// run: test_vec4_negation_zero() ~= 0.0

