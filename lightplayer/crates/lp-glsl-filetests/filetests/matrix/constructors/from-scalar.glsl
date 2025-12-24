// test run
// target riscv32.fixed32

// ============================================================================
// Matrix constructor from single scalar (identity matrix)
// mat2(scalar) creates identity matrix with diagonal = scalar, rest = 0.0
// ============================================================================

float test_mat2_from_scalar_1() {
    mat2 m = mat2(1.0);
    // Identity matrix: [1.0, 0.0; 0.0, 1.0] (column-major)
    // m[0][0]=1.0, m[1][0]=0.0, m[0][1]=0.0, m[1][1]=1.0
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1.0 + 0.0 + 0.0 + 1.0 = 2.0
}

// run: test_mat2_from_scalar_1() ~= 2.0

float test_mat2_from_scalar_2() {
    mat2 m = mat2(2.0);
    // Identity matrix scaled by 2: [2.0, 0.0; 0.0, 2.0]
    return m[0][0] + m[1][1];
    // Should be 2.0 + 2.0 = 4.0
}

// run: test_mat2_from_scalar_2() ~= 4.0

float test_mat3_from_scalar_1() {
    mat3 m = mat3(1.0);
    // Identity matrix: diagonal = 1.0, rest = 0.0
    return m[0][0] + m[1][1] + m[2][2];
    // Should be 1.0 + 1.0 + 1.0 = 3.0
}

// run: test_mat3_from_scalar_1() ~= 3.0

float test_mat3_from_scalar_3() {
    mat3 m = mat3(3.0);
    // Identity matrix scaled by 3: diagonal = 3.0, rest = 0.0
    return m[0][0] + m[1][1] + m[2][2];
    // Should be 3.0 + 3.0 + 3.0 = 9.0
}

// run: test_mat3_from_scalar_3() ~= 9.0

float test_mat3_from_scalar_off_diagonal() {
    mat3 m = mat3(5.0);
    // Off-diagonal elements should be 0.0
    return m[0][1] + m[0][2] + m[1][0] + m[1][2] + m[2][0] + m[2][1];
    // Should be 0.0 + 0.0 + 0.0 + 0.0 + 0.0 + 0.0 = 0.0
}

// run: test_mat3_from_scalar_off_diagonal() ~= 0.0

float test_mat4_from_scalar_1() {
    mat4 m = mat4(1.0);
    // Identity matrix: diagonal = 1.0, rest = 0.0
    return m[0][0] + m[1][1] + m[2][2] + m[3][3];
    // Should be 1.0 + 1.0 + 1.0 + 1.0 = 4.0
}

// run: test_mat4_from_scalar_1() ~= 4.0

float test_mat4_from_scalar_4() {
    mat4 m = mat4(4.0);
    // Identity matrix scaled by 4: diagonal = 4.0, rest = 0.0
    return m[0][0] + m[1][1] + m[2][2] + m[3][3];
    // Should be 4.0 + 4.0 + 4.0 + 4.0 = 16.0
}

// run: test_mat4_from_scalar_4() ~= 16.0

