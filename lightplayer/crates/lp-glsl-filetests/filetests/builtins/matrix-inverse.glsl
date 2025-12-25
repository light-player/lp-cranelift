// test run
// target riscv32.fixed32

// ============================================================================
// inverse(): Matrix inverse function
// inverse(mat) - returns inverse of matrix
// Undefined if matrix is singular or poorly-conditioned
// ============================================================================

mat2 test_inverse_mat2_identity() {
    // inverse of 2x2 identity matrix
    mat2 m = mat2(1.0, 0.0, 0.0, 1.0);
    return inverse(m);
}

// run: test_inverse_mat2_identity() ~= mat2(1.0, 0.0, 0.0, 1.0)

mat2 test_inverse_mat2_simple() {
    // inverse of 2x2 matrix [1,2; 3,4]
    // inverse = 1/(-2) * [-4, 2; -3, 1] = [-2, 1; -1.5, 0.5]
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return inverse(m);
}

// run: test_inverse_mat2_simple() ~= mat2(-2.0, 1.0, -1.5, 0.5)

mat3 test_inverse_mat3_identity() {
    // inverse of 3x3 identity matrix
    mat3 m = mat3(1.0);
    return inverse(m);
}

// run: test_inverse_mat3_identity() ~= mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0)

mat4 test_inverse_mat4_identity() {
    // inverse of 4x4 identity matrix
    mat4 m = mat4(1.0);
    return inverse(m);
}

// run: test_inverse_mat4_identity() ~= mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)

mat2 test_inverse_mat2_scaling() {
    // inverse of scaling matrix [2,0; 0,3] should be [0.5,0; 0,0.333]
    mat2 m = mat2(2.0, 0.0, 0.0, 3.0);
    return inverse(m);
}

// run: test_inverse_mat2_scaling() ~= mat2(0.5, 0.0, 0.0, 0.3333333333333333)
