// test run
// target riscv32.fixed32

// ============================================================================
// determinant(): Matrix determinant function
// determinant(mat) - returns determinant of matrix
// ============================================================================

float test_determinant_mat2_identity() {
    // determinant of 2x2 identity matrix
    mat2 m = mat2(1.0, 0.0, 0.0, 1.0);
    return determinant(m);
}

// run: test_determinant_mat2_identity() ~= 1.0

float test_determinant_mat2_simple() {
    // determinant of 2x2 matrix: det([1,2; 3,4]) = 1*4 - 2*3 = -2
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return determinant(m);
}

// run: test_determinant_mat2_simple() ~= -2.0

float test_determinant_mat2_zeros() {
    // determinant of singular 2x2 matrix
    mat2 m = mat2(1.0, 2.0, 2.0, 4.0);
    return determinant(m);
}

// run: test_determinant_mat2_zeros() ~= 0.0

float test_determinant_mat3_identity() {
    // determinant of 3x3 identity matrix
    mat3 m = mat3(1.0);
    return determinant(m);
}

// run: test_determinant_mat3_identity() ~= 1.0

float test_determinant_mat3_simple() {
    // determinant of 3x3 matrix
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return determinant(m);
}

// run: test_determinant_mat3_simple() ~= 0.0

float test_determinant_mat4_identity() {
    // determinant of 4x4 identity matrix
    mat4 m = mat4(1.0);
    return determinant(m);
}

// run: test_determinant_mat4_identity() ~= 1.0
