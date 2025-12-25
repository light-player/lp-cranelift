// test run
// target riscv32.fixed32

// ============================================================================
// matrixCompMult(): Matrix component-wise multiply function
// matrixCompMult(mat, mat) - component-wise matrix multiply
// ============================================================================

mat2 test_matrixcompmult_mat2_identity() {
    // matrixCompMult with identity matrices
    mat2 a = mat2(1.0, 0.0, 0.0, 1.0);
    mat2 b = mat2(1.0, 0.0, 0.0, 1.0);
    return matrixCompMult(a, b);
}

// run: test_matrixcompmult_mat2_identity() ~= mat2(1.0, 0.0, 0.0, 1.0)

mat2 test_matrixcompmult_mat2_simple() {
    // matrixCompMult with simple 2x2 matrices
    mat2 a = mat2(2.0, 3.0, 4.0, 5.0);
    mat2 b = mat2(1.0, 2.0, 3.0, 4.0);
    return matrixCompMult(a, b);
}

// run: test_matrixcompmult_mat2_simple() ~= mat2(2.0, 6.0, 12.0, 20.0)

mat3 test_matrixcompmult_mat3() {
    // matrixCompMult with 3x3 matrices
    mat3 a = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 b = mat3(2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}

// run: test_matrixcompmult_mat3() ~= mat3(2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0, 18.0)

mat4 test_matrixcompmult_mat4() {
    // matrixCompMult with 4x4 matrices
    mat4 a = mat4(1.0);
    mat4 b = mat4(3.0);
    return matrixCompMult(a, b);
}

// run: test_matrixcompmult_mat4() ~= mat4(3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0, 3.0)

mat2 test_matrixcompmult_mat2_zeros() {
    // matrixCompMult resulting in zeros
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(0.0, 0.0, 0.0, 0.0);
    return matrixCompMult(a, b);
}

// run: test_matrixcompmult_mat2_zeros() ~= mat2(0.0, 0.0, 0.0, 0.0)
