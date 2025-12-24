// test run
// target riscv32.fixed32

// ============================================================================
// Matrix negation: -m (component-wise)
// ============================================================================

float test_mat2_negation() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 result = -m;
    // Component-wise negation
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be -1.0 + -2.0 + -3.0 + -4.0 = -10.0
}

// run: test_mat2_negation() ~= -10.0

float test_mat3_negation() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 result = -m;
    // Sum diagonal
    return result[0][0] + result[1][1] + result[2][2];
    // Should be -1.0 + -5.0 + -9.0 = -15.0
}

// run: test_mat3_negation() ~= -15.0

float test_mat4_negation() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    mat4 result = -m;
    // Sum column 0 (all rows of column 0)
    return result[0][0] + result[0][1] + result[0][2] + result[0][3];
    // Should be -1.0 + -5.0 + -9.0 + -13.0 = -28.0
}

// run: test_mat4_negation() ~= -28.0

