// test run
// target riscv32.fixed32

// ============================================================================
// Matrix multiplication
// Matrix * Matrix → Matrix (matrix multiplication, not component-wise)
// Matrix * Vector → Vector
// Vector * Matrix → Vector
// Scalar * Matrix → Matrix (component-wise)
// Matrix * Scalar → Matrix (component-wise)
// ============================================================================

float test_mat2_mat2_multiplication() {
    mat2 m1 = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 m2 = mat2(5.0, 6.0, 7.0, 8.0);
    // Column-major: m1 = [1.0, 3.0; 2.0, 4.0], m2 = [5.0, 7.0; 6.0, 8.0]
    mat2 result = m1 * m2;
    // Matrix multiplication (not component-wise)
    // result[0][0] (col 0, row 0) = m1[0][0]*m2[0][0] + m1[1][0]*m2[0][1] = 1.0*5.0 + 3.0*6.0 = 23.0
    // result[0][1] (col 0, row 1) = m1[0][1]*m2[0][0] + m1[1][1]*m2[0][1] = 2.0*5.0 + 4.0*6.0 = 34.0
    // result[1][0] (col 1, row 0) = m1[0][0]*m2[1][0] + m1[1][0]*m2[1][1] = 1.0*7.0 + 3.0*8.0 = 31.0
    // result[1][1] (col 1, row 1) = m1[0][1]*m2[1][0] + m1[1][1]*m2[1][1] = 2.0*7.0 + 4.0*8.0 = 46.0
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 23.0 + 31.0 + 34.0 + 46.0 = 134.0
}

// run: test_mat2_mat2_multiplication() ~= 134.0

float test_mat2_vec2_multiplication() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    vec2 v = vec2(5.0, 6.0);
    vec2 result = m * v;
    // Matrix * Vector
    // m = [1.0, 3.0; 2.0, 4.0], v = [5.0, 6.0]
    // result.x = 1.0*5.0 + 3.0*6.0 = 5.0 + 18.0 = 23.0
    // result.y = 2.0*5.0 + 4.0*6.0 = 10.0 + 24.0 = 34.0
    return result.x + result.y;
    // Should be 23.0 + 34.0 = 57.0
}

// run: test_mat2_vec2_multiplication() ~= 57.0

float test_vec2_mat2_multiplication() {
    vec2 v = vec2(1.0, 2.0);
    mat2 m = mat2(3.0, 4.0, 5.0, 6.0);
    vec2 result = v * m;
    // Vector * Matrix
    // v = [1.0, 2.0], m = [3.0, 5.0; 4.0, 6.0]
    // result.x = 1.0*3.0 + 2.0*4.0 = 3.0 + 8.0 = 11.0
    // result.y = 1.0*5.0 + 2.0*6.0 = 5.0 + 12.0 = 17.0
    return result.x + result.y;
    // Should be 11.0 + 17.0 = 28.0
}

// run: test_vec2_mat2_multiplication() ~= 28.0

float test_scalar_mat2_multiplication() {
    float s = 2.0;
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    mat2 result = s * m;
    // Scalar * Matrix (component-wise)
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 2.0 + 6.0 + 4.0 + 8.0 = 20.0
}

// run: test_scalar_mat2_multiplication() ~= 20.0

float test_mat2_scalar_multiplication() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    float s = 3.0;
    mat2 result = m * s;
    // Matrix * Scalar (component-wise)
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 3.0 + 9.0 + 6.0 + 12.0 = 30.0
}

// run: test_mat2_scalar_multiplication() ~= 30.0

float test_mat3_vec3_multiplication() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    vec3 v = vec3(1.0, 2.0, 3.0);
    vec3 result = m * v;
    // Matrix * Vector
    // First element: 1.0*1.0 + 4.0*2.0 + 7.0*3.0 = 1.0 + 8.0 + 21.0 = 30.0
    return result.x;
    // Should be 30.0
}

// run: test_mat3_vec3_multiplication() ~= 30.0

