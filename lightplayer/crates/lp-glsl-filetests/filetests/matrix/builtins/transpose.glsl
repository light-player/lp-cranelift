// test run
// target riscv32.fixed32

// ============================================================================
// Matrix transpose: transpose(m)
// transpose(m)[col][row] == m[row][col]
// GLSL spec: m[col][row] - first index is column, second is row
// ============================================================================

float test_mat2_transpose() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    // m = [1.0, 3.0; 2.0, 4.0] (column-major)
    mat2 t = transpose(m);
    // transpose(m) = [1.0, 2.0; 3.0, 4.0] (column-major)
    // Column 0: [1.0, 3.0] → t[0][0]=1.0, t[0][1]=3.0
    // Column 1: [2.0, 4.0] → t[1][0]=2.0, t[1][1]=4.0
    return t[0][0] + t[1][0] + t[0][1] + t[1][1];
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_mat2_transpose() ~= 10.0

float test_mat2_transpose_verify() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    // Column-major: Column 0: [10.0, 20.0], Column 1: [30.0, 40.0]
    mat2 t = transpose(m);
    // Verify: t[0][1] (col 0, row 1) should equal m[1][0] (col 1, row 0)
    return t[0][1] + m[1][0];
    // Should be 30.0 + 30.0 = 60.0
}

// run: test_mat2_transpose_verify() ~= 60.0

float test_mat3_transpose() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 t = transpose(m);
    // Verify diagonal (should be unchanged)
    return t[0][0] + t[1][1] + t[2][2];
    // Should be 1.0 + 5.0 + 9.0 = 15.0
}

// run: test_mat3_transpose() ~= 15.0

float test_mat3_transpose_verify() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 t = transpose(m);
    // Verify: t[1][2] should equal m[2][1]
    return t[1][2] + m[2][1];
    // Should be 6.0 + 6.0 = 12.0
}

// run: test_mat3_transpose_verify() ~= 12.0

float test_mat4_transpose() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    mat4 t = transpose(m);
    // Verify: t[2][3] should equal m[3][2]
    return t[2][3] + m[3][2];
    // Should be 14.0 + 14.0 = 28.0
}

// run: test_mat4_transpose() ~= 28.0

