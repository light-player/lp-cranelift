// test run
// target riscv32.fixed32

// ============================================================================
// Vector multiplication: vec4 * vec4, vec4 * float, vec4 * mat4
// ============================================================================

float test_vec4_multiplication() {
    vec4 v1 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 v2 = vec4(5.0, 6.0, 7.0, 8.0);
    vec4 result = v1 * v2;
    // Component-wise multiplication
    return result.x + result.y + result.z + result.w;
    // Should be 5.0 + 12.0 + 21.0 + 32.0 = 70.0
}

// run: test_vec4_multiplication() ~= 70.0

float test_vec4_multiplication_scalar() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float s = 5.0;
    vec4 result = v * s;
    // Scalar multiplication (component-wise)
    return result.x + result.y + result.z + result.w;
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_vec4_multiplication_scalar() ~= 50.0

float test_vec4_multiplication_mat4() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    mat4 m = mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, 2.0, 0.0, 0.0,
        0.0, 0.0, 3.0, 0.0,
        0.0, 0.0, 0.0, 4.0
    );
    vec4 result = m * v;
    // Matrix * Vector (linear algebraic multiply)
    // Diagonal matrix multiplies each component
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 4.0 + 9.0 + 16.0 = 30.0
}

// run: test_vec4_multiplication_mat4() ~= 30.0

float test_vec4_multiplication_verify_components() {
    vec4 v1 = vec4(2.0, 3.0, 4.0, 5.0);
    vec4 v2 = vec4(6.0, 7.0, 8.0, 9.0);
    vec4 result = v1 * v2;
    // Verify each component
    float sum = 0.0;
    if (result.x == 12.0) sum = sum + 1.0;
    if (result.y == 21.0) sum = sum + 1.0;
    if (result.z == 32.0) sum = sum + 1.0;
    if (result.w == 45.0) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all components correct)
}

// run: test_vec4_multiplication_verify_components() ~= 4.0

float test_vec4_multiplication_negative() {
    vec4 v1 = vec4(-1.0, 2.0, -3.0, 4.0);
    vec4 v2 = vec4(5.0, -6.0, 7.0, -8.0);
    vec4 result = v1 * v2;
    // Mixed positive and negative
    return result.x + result.y + result.z + result.w;
    // Should be -5.0 + -12.0 + -21.0 + -32.0 = -70.0
}

// run: test_vec4_multiplication_negative() ~= -70.0

float test_vec4_multiplication_scalar_zero() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    float s = 0.0;
    vec4 result = v * s;
    // Multiply by zero
    return result.x + result.y + result.z + result.w;
    // Should be 0.0 + 0.0 + 0.0 + 0.0 = 0.0
}

// run: test_vec4_multiplication_scalar_zero() ~= 0.0

