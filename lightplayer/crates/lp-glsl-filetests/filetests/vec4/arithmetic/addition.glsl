// test run
// target riscv32.fixed32

// ============================================================================
// Vector addition: vec4 + vec4, vec4 + float (component-wise)
// ============================================================================

float test_vec4_addition() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = v1 + v2;
    // Component-wise addition
    return result.x + result.y + result.z + result.w;
    // Should be 11.0 + 22.0 + 33.0 + 44.0 = 110.0
}

// run: test_vec4_addition() ~= 110.0

float test_vec4_addition_scalar() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float s = 5.0;
    vec4 result = v + s;
    // Scalar addition (component-wise)
    return result.x + result.y + result.z + result.w;
    // Should be 6.0 + 7.0 + 8.0 + 9.0 = 30.0
}

// run: test_vec4_addition_scalar() ~= 30.0

float test_vec4_addition_verify_components() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = v1 + v2;
    // Verify each component
    float sum = 0.0;
    if (result.x == 11.0) sum = sum + 1.0;
    if (result.y == 22.0) sum = sum + 1.0;
    if (result.z == 33.0) sum = sum + 1.0;
    if (result.w == 44.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components correct)
}

// run: test_vec4_addition_verify_components() ~= 4.0

float test_vec4_addition_negative() {
    vec4 v1 = vec4(-1.0, -2.0, -3.0, -4.0);
    vec4 v2 = vec4(5.0, 6.0, 7.0, 8.0);
    vec4 result = v1 + v2;
    // Negative + positive
    return result.x + result.y + result.z + result.w;
    // Should be 4.0 + 4.0 + 4.0 + 4.0 = 16.0
}

// run: test_vec4_addition_negative() ~= 16.0

float test_vec4_addition_scalar_negative() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    float s = -5.0;
    vec4 result = v + s;
    // Vector + negative scalar
    return result.x + result.y + result.z + result.w;
    // Should be 5.0 + 15.0 + 25.0 + 35.0 = 80.0
}

// run: test_vec4_addition_scalar_negative() ~= 80.0

