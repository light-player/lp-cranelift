// test run
// target riscv32.fixed32

// ============================================================================
// Matrix constructor from mixed scalar and vector arguments
// GLSL allows mixing scalars and vectors in matrix constructors
// ============================================================================

float test_mat2_mixed_scalar_vec2() {
    // Test mixed constructor: first column from vec2, second column from scalars
    mat2 m2 = mat2(vec2(1.0, 2.0), 3.0, 4.0);
    // First column from vec2, second column from scalars
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    // m2[0][0]=1.0, m2[0][1]=2.0, m2[1][0]=3.0, m2[1][1]=4.0
    return m2[0][0] + m2[1][0] + m2[0][1] + m2[1][1];
    // Should be 1.0 + 3.0 + 2.0 + 4.0 = 10.0
}

// run: test_mat2_mixed_scalar_vec2() ~= 10.0

float test_mat3_mixed() {
    mat3 m = mat3(vec2(1.0, 2.0), 3.0, vec3(4.0, 5.0, 6.0), 7.0, 8.0, 9.0);
    // Mixed arguments fill column-major order
    // Column 0: [1.0, 2.0, 3.0] (from vec2 + scalar)
    // Column 1: [4.0, 5.0, 6.0] (from vec3)
    // Column 2: [7.0, 8.0, 9.0] (from scalars)
    // m[0][0]=1.0, m[1][0]=4.0, m[2][0]=7.0 (first element of each column)
    return m[0][0] + m[1][0] + m[2][0];
    // Should be 1.0 + 4.0 + 7.0 = 12.0
}

// run: test_mat3_mixed() ~= 12.0


