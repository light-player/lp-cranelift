// test run
// target riscv32.fixed32

// ============================================================================
// Matrix constructor from matrix (conversion between sizes)
// Smaller matrix is padded with identity, larger matrix is truncated
// ============================================================================

float test_mat3_from_mat2() {
    mat2 m2 = mat2(1.0, 2.0, 3.0, 4.0);
    mat3 m3 = mat3(m2);
    // mat3 from mat2: upper-left 2x2 from m2, rest is identity
    // [1.0, 3.0, 0.0; 2.0, 4.0, 0.0; 0.0, 0.0, 1.0]
    return m3[0][0] + m3[1][1] + m3[2][2];
    // Should be 1.0 + 4.0 + 1.0 = 6.0
}

// run: test_mat3_from_mat2() ~= 6.0

float test_mat3_from_mat2_verify_padding() {
    mat2 m2 = mat2(10.0, 20.0, 30.0, 40.0);
    mat3 m3 = mat3(m2);
    // Verify padding: m3[2][0], m3[0][2], m3[1][2], m3[2][1] should be 0.0
    // m3[2][2] should be 1.0 (identity)
    return m3[2][0] + m3[0][2] + m3[1][2] + m3[2][1] + m3[2][2];
    // Should be 0.0 + 0.0 + 0.0 + 0.0 + 1.0 = 1.0
}

// run: test_mat3_from_mat2_verify_padding() ~= 1.0

float test_mat2_from_mat3() {
    mat3 m3 = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat2 m2 = mat2(m3);
    // mat2 from mat3: takes upper-left 2x2
    // [1.0, 4.0; 2.0, 5.0]
    return m2[0][0] + m2[1][0] + m2[0][1] + m2[1][1];
    // Should be 1.0 + 2.0 + 4.0 + 5.0 = 12.0
}

// run: test_mat2_from_mat3() ~= 12.0

float test_mat4_from_mat2() {
    mat2 m2 = mat2(1.0, 2.0, 3.0, 4.0);
    mat4 m4 = mat4(m2);
    // mat4 from mat2: upper-left 2x2 from m2, rest is identity
    // Diagonal should be: 1.0, 4.0, 1.0, 1.0
    return m4[0][0] + m4[1][1] + m4[2][2] + m4[3][3];
    // Should be 1.0 + 4.0 + 1.0 + 1.0 = 7.0
}

// run: test_mat4_from_mat2() ~= 7.0

float test_mat4_from_mat3() {
    mat3 m3 = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat4 m4 = mat4(m3);
    // mat4 from mat3: upper-left 3x3 from m3, rest is identity
    // Diagonal should be: 1.0, 5.0, 9.0, 1.0
    return m4[0][0] + m4[1][1] + m4[2][2] + m4[3][3];
    // Should be 1.0 + 5.0 + 9.0 + 1.0 = 16.0
}

// run: test_mat4_from_mat3() ~= 16.0


