// test run
// target riscv32.fixed32

// ============================================================================
// Vector subtraction: vec4 - vec4, vec4 - float (component-wise)
// ============================================================================

float test_vec4_subtraction() {
    vec4 v1 = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 v2 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v1 - v2;
    // Component-wise subtraction
    return result.x + result.y + result.z + result.w;
    // Should be 9.0 + 18.0 + 27.0 + 36.0 = 90.0
}

// run: test_vec4_subtraction() ~= 90.0

float test_vec4_subtraction_scalar() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    float s = 5.0;
    vec4 result = v - s;
    // Scalar subtraction (component-wise)
    return result.x + result.y + result.z + result.w;
    // Should be 5.0 + 15.0 + 25.0 + 35.0 = 80.0
}

// run: test_vec4_subtraction_scalar() ~= 80.0

float test_vec4_subtraction_verify_components() {
    vec4 v1 = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 v2 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 result = v1 - v2;
    // Verify each component
    float sum = 0.0;
    if (result.x == 9.0) sum = sum + 1.0;
    if (result.y == 18.0) sum = sum + 1.0;
    if (result.z == 27.0) sum = sum + 1.0;
    if (result.w == 36.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components correct)
}

// run: test_vec4_subtraction_verify_components() ~= 4.0

float test_vec4_subtraction_negative() {
    vec4 v1 = vec4(5.0, 6.0, 7.0, 8.0);
    vec4 v2 = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 result = v1 - v2;
    // Smaller - larger (results in negative)
    return result.x + result.y + result.z + result.w;
    // Should be -5.0 + -14.0 + -23.0 + -32.0 = -74.0
}

// run: test_vec4_subtraction_negative() ~= -74.0

float test_vec4_subtraction_scalar_negative() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    float s = 15.0;
    vec4 result = v - s;
    // Vector - larger scalar
    return result.x + result.y + result.z + result.w;
    // Should be -5.0 + 5.0 + 15.0 + 25.0 = 40.0
}

// run: test_vec4_subtraction_scalar_negative() ~= 40.0

