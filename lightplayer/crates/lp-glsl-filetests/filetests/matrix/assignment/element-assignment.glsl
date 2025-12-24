// test run
// target riscv32.fixed32

// ============================================================================
// Element assignment: m[col][row] = value
// GLSL spec: m[col][row] - first index is column, second is row
// ============================================================================

float test_mat2_element_assignment() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    m[0][1] = 100.0;  // Assign to col 0, row 1
    // After: m[0][0]=1.0, m[0][1]=100.0, m[1][0]=3.0, m[1][1]=4.0
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1.0 + 3.0 + 100.0 + 4.0 = 108.0
}

// run: test_mat2_element_assignment() ~= 108.0

float test_mat2_element_assignment_verify_others() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    // Column-major: Column 0: [10.0, 20.0], Column 1: [30.0, 40.0]
    m[1][0] = 200.0;  // Assign to col 1, row 0
    // Verify other elements unchanged
    return m[0][0] + m[0][1] + m[1][1];
    // Should be 10.0 + 20.0 + 40.0 = 70.0
}

// run: test_mat2_element_assignment_verify_others() ~= 70.0

float test_mat3_element_assignment() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    // Column-major: Column 0: [1,2,3], Column 1: [4,5,6], Column 2: [7,8,9]
    m[1][2] = 500.0;  // Assign to col 1, row 2
    return m[1][2];
    // Should be 500.0
}

// run: test_mat3_element_assignment() ~= 500.0

float test_mat4_element_assignment() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Column-major: Column 0: [1,5,9,13], Column 1: [2,6,10,14], Column 2: [3,7,11,15], Column 3: [4,8,12,16]
    m[2][3] = 1000.0;  // Assign to col 2, row 3
    return m[2][3];
    // Should be 1000.0
}

// run: test_mat4_element_assignment() ~= 1000.0

