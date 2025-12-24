// test run
// target riscv32.fixed32

// ============================================================================
// Nested matrix indexing patterns
// Complex expressions using matrix indexing
// ============================================================================

float test_mat2_nested_indexing() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Access element through column access
    float val = m[0].x + m[0].y + m[1].x + m[1].y;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
    return val;
}

// run: test_mat2_nested_indexing() ~= 10.0

float test_mat2_chained_access() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    // Chain column access with component access
    float sum = m[0].x + m[1].y;
    // Should be 10.0 + 40.0 = 50.0
    return sum;
}

// run: test_mat2_chained_access() ~= 50.0

float test_mat2_expression_indexing() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    // Use indexing in expressions
    float result = m[0][0] + m[0][1] + m[1][0] + m[1][1];
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
    return result;
}

// run: test_mat2_expression_indexing() ~= 10.0

float test_mat3_nested_indexing() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    // Access through column and component
    float val = m[1].y + m[2].z;
    // Column 1, row 1 (y component) = 5.0
    // Column 2, row 2 (z component) = 9.0
    // Should be 5.0 + 9.0 = 14.0
    return val;
}

// run: test_mat3_nested_indexing() ~= 14.0

float test_mat4_nested_indexing() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Complex nested access
    float val = m[0].x + m[1].y + m[2].z + m[3].w;
    // Should be 1.0 + 6.0 + 11.0 + 16.0 = 34.0
    return val;
}

// run: test_mat4_nested_indexing() ~= 34.0

