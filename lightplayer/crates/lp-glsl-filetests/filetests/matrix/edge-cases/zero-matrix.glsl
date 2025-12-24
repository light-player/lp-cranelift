// test run
// target riscv32.fixed32

// ============================================================================
// Zero matrix tests
// ============================================================================

float test_mat2_zero() {
    mat2 m = mat2(0.0, 0.0, 0.0, 0.0);
    // All elements should be 0.0
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 0.0
}

// run: test_mat2_zero() ~= 0.0

float test_mat2_zero_addition() {
    mat2 m1 = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 m2 = mat2(0.0, 0.0, 0.0, 0.0);
    mat2 result = m1 + m2;
    // Adding zero should not change m1
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_mat2_zero_addition() ~= 10.0

float test_mat2_zero_multiplication() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 zero = mat2(0.0, 0.0, 0.0, 0.0);
    mat2 result = m * zero;
    // Matrix multiplication with zero matrix should give zero matrix
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 0.0
}

// run: test_mat2_zero_multiplication() ~= 0.0

float test_mat3_zero() {
    mat3 m = mat3(0.0);
    // All elements should be 0.0
    return m[0][0] + m[1][1] + m[2][2];
    // Should be 0.0
}

// run: test_mat3_zero() ~= 0.0

float test_mat4_zero() {
    mat4 m = mat4(0.0);
    // All elements should be 0.0
    return m[0][0] + m[1][1] + m[2][2] + m[3][3];
    // Should be 0.0
}

// run: test_mat4_zero() ~= 0.0


