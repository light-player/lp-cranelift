// test run
// target riscv32.fixed32

// ============================================================================
// Matrix constructor from column vectors
// mat2(vec2, vec2) - each vec2 is a column (column-major order)
// ============================================================================

float test_mat2_from_vec2() {
    mat2 m = mat2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    // Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    // Matrix: [1.0, 3.0; 2.0, 4.0] (column-major)
    // m[0][0]=1.0 (col 0, row 0), m[0][1]=2.0 (col 0, row 1)
    // m[1][0]=3.0 (col 1, row 0), m[1][1]=4.0 (col 1, row 1)
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1.0 + 3.0 + 2.0 + 4.0 = 10.0
}

// run: test_mat2_from_vec2() ~= 10.0

float test_mat2_from_vec2_verify_order() {
    mat2 m = mat2(vec2(10.0, 20.0), vec2(30.0, 40.0));
    // Verify column-major order: first vec2 is column 0
    // Column 0: [10.0, 20.0] → m[0][0]=10.0, m[0][1]=20.0
    // Column 1: [30.0, 40.0] → m[1][0]=30.0, m[1][1]=40.0
    float col0_sum = m[0][0] + m[0][1];  // Should be 10.0 + 20.0 = 30.0
    float col1_sum = m[1][0] + m[1][1];  // Should be 30.0 + 40.0 = 70.0
    return col0_sum + col1_sum;
    // Should be 30.0 + 70.0 = 100.0
}

// run: test_mat2_from_vec2_verify_order() ~= 100.0

float test_mat3_from_vec3() {
    mat3 m = mat3(
        vec3(1.0, 2.0, 3.0),
        vec3(4.0, 5.0, 6.0),
        vec3(7.0, 8.0, 9.0)
    );
    // Column 0: [1.0, 2.0, 3.0], Column 1: [4.0, 5.0, 6.0], Column 2: [7.0, 8.0, 9.0]
    // Verify first column (col 0)
    return m[0][0] + m[0][1] + m[0][2];
    // Should be 1.0 + 2.0 + 3.0 = 6.0
}

// run: test_mat3_from_vec3() ~= 6.0

float test_mat3_from_vec3_all_columns() {
    mat3 m = mat3(
        vec3(1.0, 2.0, 3.0),
        vec3(4.0, 5.0, 6.0),
        vec3(7.0, 8.0, 9.0)
    );
    // Sum all elements (column-major order)
    return m[0][0] + m[0][1] + m[0][2] +  // Column 0
           m[1][0] + m[1][1] + m[1][2] +  // Column 1
           m[2][0] + m[2][1] + m[2][2];  // Column 2
    // Should be 1+2+3+4+5+6+7+8+9 = 45.0
}

// run: test_mat3_from_vec3_all_columns() ~= 45.0

float test_mat4_from_vec4() {
    mat4 m = mat4(
        vec4(1.0, 2.0, 3.0, 4.0),
        vec4(5.0, 6.0, 7.0, 8.0),
        vec4(9.0, 10.0, 11.0, 12.0),
        vec4(13.0, 14.0, 15.0, 16.0)
    );
    // Column 0: [1.0, 2.0, 3.0, 4.0]
    // Verify first column (col 0)
    return m[0][0] + m[0][1] + m[0][2] + m[0][3];
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_mat4_from_vec4() ~= 10.0

float test_mat4_from_vec4_column_2() {
    mat4 m = mat4(
        vec4(1.0, 2.0, 3.0, 4.0),
        vec4(5.0, 6.0, 7.0, 8.0),
        vec4(9.0, 10.0, 11.0, 12.0),
        vec4(13.0, 14.0, 15.0, 16.0)
    );
    // Column 2: [9.0, 10.0, 11.0, 12.0]
    // Verify column 2 (third column, col index 2)
    return m[2][0] + m[2][1] + m[2][2] + m[2][3];
    // Should be 9.0 + 10.0 + 11.0 + 12.0 = 42.0
}

// run: test_mat4_from_vec4_column_2() ~= 42.0

