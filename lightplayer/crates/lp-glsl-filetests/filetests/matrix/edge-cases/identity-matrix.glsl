// test run
// target riscv32.fixed32

// ============================================================================
// Identity matrix tests
// Identity matrix: diagonal = 1.0, rest = 0.0
// ============================================================================

float test_mat2_identity_construction() {
    mat2 m = mat2(1.0);
    // Identity matrix: [1.0, 0.0; 0.0, 1.0]
    return m[0][0] + m[1][1];
    // Should be 1.0 + 1.0 = 2.0
}

// run: test_mat2_identity_construction() ~= 2.0

float test_mat2_identity_off_diagonal() {
    mat2 m = mat2(1.0);
    // Off-diagonal elements should be 0.0
    return m[0][1] + m[1][0];
    // Should be 0.0 + 0.0 = 0.0
}

// run: test_mat2_identity_off_diagonal() ~= 0.0

float test_mat2_identity_multiplication() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    mat2 identity = mat2(1.0);
    mat2 result = m * identity;
    // m * identity should equal m
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 1.0 + 3.0 + 2.0 + 4.0 = 10.0
}

// run: test_mat2_identity_multiplication() ~= 10.0

float test_mat2_identity_determinant() {
    mat2 m = mat2(1.0);
    float det = determinant(m);
    return det;
    // Should be 1.0
}

// run: test_mat2_identity_determinant() ~= 1.0

float test_mat3_identity() {
    mat3 m = mat3(1.0);
    // Verify diagonal
    return m[0][0] + m[1][1] + m[2][2];
    // Should be 1.0 + 1.0 + 1.0 = 3.0
}

// run: test_mat3_identity() ~= 3.0

float test_mat4_identity() {
    mat4 m = mat4(1.0);
    // Verify diagonal
    return m[0][0] + m[1][1] + m[2][2] + m[3][3];
    // Should be 1.0 + 1.0 + 1.0 + 1.0 = 4.0
}

// run: test_mat4_identity() ~= 4.0

