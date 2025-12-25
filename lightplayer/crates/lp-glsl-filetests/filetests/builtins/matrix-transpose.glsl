// test run
// target riscv32.fixed32

// ============================================================================
// transpose(): Matrix transpose function
// transpose(mat) - returns transpose of matrix
// ============================================================================

mat2 test_transpose_mat2() {
    // transpose of 2x2 matrix
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return transpose(m);
}

// run: test_transpose_mat2() ~= mat2(1.0, 3.0, 2.0, 4.0)

mat3 test_transpose_mat3() {
    // transpose of 3x3 matrix
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose(m);
}

// run: test_transpose_mat3() ~= mat3(1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0)

mat4 test_transpose_mat4() {
    // transpose of 4x4 identity matrix
    mat4 m = mat4(1.0);
    return transpose(m);
}

// run: test_transpose_mat4() ~= mat4(1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0)

mat2x3 test_transpose_mat2x3() {
    // transpose of 2x3 matrix (becomes 3x2)
    mat2x3 m = mat2x3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    return transpose(m);
}

// run: test_transpose_mat2x3() ~= mat3x2(1.0, 4.0, 2.0, 5.0, 3.0, 6.0)

mat3x2 test_transpose_mat3x2() {
    // transpose of 3x2 matrix (becomes 2x3)
    mat3x2 m = mat3x2(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
    return transpose(m);
}

// run: test_transpose_mat3x2() ~= mat2x3(1.0, 3.0, 5.0, 2.0, 4.0, 6.0)
