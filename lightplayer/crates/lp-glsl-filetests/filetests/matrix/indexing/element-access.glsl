// test run
// target riscv32.fixed32

// ============================================================================
// Matrix element access: m[col][row]
// Tests reading and writing individual matrix elements
// Critical: Verify column-major order indexing
// GLSL spec: m[col][row] - first index is column, second is row
// ============================================================================

float test_mat2_element_read() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 3.0], Column 1: [2.0, 4.0]
    // m[0][0]=1.0 (col 0, row 0), m[0][1]=3.0 (col 0, row 1)
    // m[1][0]=2.0 (col 1, row 0), m[1][1]=4.0 (col 1, row 1)
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_mat2_element_read() ~= 10.0

float test_mat2_element_write() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    m[0][1] = 10.0;  // Write to element at col 0, row 1
    // After write: m[0][0]=1.0, m[0][1]=10.0, m[1][0]=2.0, m[1][1]=4.0
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1.0 + 2.0 + 10.0 + 4.0 = 17.0
}

// run: test_mat2_element_write() ~= 17.0

float test_mat2_all_elements() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    // Verify all element positions (col, row)
    float e00 = m[0][0];  // col 0, row 0 = 10.0
    float e01 = m[0][1];  // col 0, row 1 = 20.0
    float e10 = m[1][0];  // col 1, row 0 = 30.0
    float e11 = m[1][1];  // col 1, row 1 = 40.0
    return e00 + e10 + e01 + e11;
    // Should be 10.0 + 30.0 + 20.0 + 40.0 = 100.0
}

// run: test_mat2_all_elements() ~= 100.0

float test_mat3_element_access() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    // Column-major order: Column 0: [1,2,3], Column 1: [4,5,6], Column 2: [7,8,9]
    // Column 0: m[0][0]=1.0, m[0][1]=2.0, m[0][2]=3.0
    // Column 1: m[1][0]=4.0, m[1][1]=5.0, m[1][2]=6.0
    // Column 2: m[2][0]=7.0, m[2][1]=8.0, m[2][2]=9.0
    return m[0][0] + m[1][1] + m[2][2];  // Diagonal (col i, row i)
    // Should be 1.0 + 5.0 + 9.0 = 15.0
}

// run: test_mat3_element_access() ~= 15.0

float test_mat3_element_write() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    m[1][2] = 100.0;  // Write to col 1, row 2
    // After: m[1][2] should be 100.0 (col 1, row 2)
    return m[1][2];
    // Should be 100.0
}

// run: test_mat3_element_write() ~= 100.0

float test_mat4_element_access() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Verify diagonal elements
    return m[0][0] + m[1][1] + m[2][2] + m[3][3];
    // Should be 1.0 + 6.0 + 11.0 + 16.0 = 34.0
}

// run: test_mat4_element_access() ~= 34.0

float test_mat4_element_write() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    m[2][3] = 200.0;  // Write to col 2, row 3
    return m[2][3];
    // Should be 200.0
}

// run: test_mat4_element_write() ~= 200.0

