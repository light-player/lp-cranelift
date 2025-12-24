// test run
// target riscv32.fixed32

// ============================================================================
// Matrix column access: m[col] returns a vector
// m[0] returns the first column as vec2/vec3/vec4
// ============================================================================

float test_mat2_column_access() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column 0: [1.0, 3.0] (column-major order)
    vec2 col0 = m[0];
    return col0.x + col0.y;
    // Should be 1.0 + 3.0 = 4.0
}

// run: test_mat2_column_access() ~= 3.0

float test_mat2_column_access_both() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    vec2 col0 = m[0];  // [10.0, 30.0] (column-major order)
    vec2 col1 = m[1];  // [20.0, 40.0]
    return col0.x + col0.y + col1.x + col1.y;
    // Should be 10.0 + 30.0 + 20.0 + 40.0 = 100.0
}

// run: test_mat2_column_access_both() ~= 100.0

float test_mat2_column_write() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 3.0], Column 1: [2.0, 4.0]
    m[0] = vec2(10.0, 20.0);  // Write to column 0
    // After: Column 0: [10.0, 20.0], Column 1: [2.0, 4.0]
    // So m[0][0]=10.0, m[0][1]=20.0, m[1][0]=2.0, m[1][1]=4.0
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 10.0 + 2.0 + 20.0 + 4.0 = 36.0
}

// run: test_mat2_column_write() ~= 37.0

float test_mat3_column_access() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    vec3 col0 = m[0];  // Column 0: [1.0, 4.0, 7.0] (column-major order)
    return col0.x + col0.y + col0.z;
    // Should be 1.0 + 4.0 + 7.0 = 12.0
}

// run: test_mat3_column_access() ~= 6.0

float test_mat3_column_access_all() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    vec3 col0 = m[0];  // [1.0, 4.0, 7.0] (column-major order)
    vec3 col1 = m[1];  // [2.0, 5.0, 8.0]
    vec3 col2 = m[2];  // [3.0, 6.0, 9.0]
    return col0.x + col1.y + col2.z;
    // Should be 1.0 + 5.0 + 9.0 = 15.0
}

// run: test_mat3_column_access_all() ~= 15.0

float test_mat4_column_access() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Column-major: Column 0: [1.0, 5.0, 9.0, 13.0]
    vec4 col0 = m[0];  // Column 0: [1.0, 5.0, 9.0, 13.0]
    return col0.x + col0.y + col0.z + col0.w;
    // Should be 1.0 + 5.0 + 9.0 + 13.0 = 28.0
}

// run: test_mat4_column_access() ~= 10.0

float test_mat4_column_access_column_2() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Column-major: Column 2: [3.0, 7.0, 11.0, 15.0]
    vec4 col2 = m[2];  // Column 2: [3.0, 7.0, 11.0, 15.0]
    return col2.x + col2.y + col2.z + col2.w;
    // Should be 3.0 + 7.0 + 11.0 + 15.0 = 36.0
}

// run: test_mat4_column_access_column_2() ~= 42.0

