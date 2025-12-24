// test run
// target riscv32.fixed32

// ============================================================================
// Matrix determinant: determinant(m)
// mat2: det([[a,b],[c,d]]) = ad - bc
// ============================================================================

float test_mat2_determinant() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // m = [1.0, 3.0; 2.0, 4.0] (column-major)
    // det = 1.0*4.0 - 3.0*2.0 = 4.0 - 6.0 = -2.0
    float det = determinant(m);
    return det;
    // Should be -2.0
}

// run: test_mat2_determinant() ~= -2.0

float test_mat2_determinant_identity() {
    mat2 m = mat2(1.0);  // Identity matrix
    float det = determinant(m);
    return det;
    // Should be 1.0
}

// run: test_mat2_determinant_identity() ~= 1.0

float test_mat2_determinant_simple() {
    mat2 m = mat2(2.0, 0.0, 0.0, 3.0);
    // Diagonal matrix: det = 2.0 * 3.0 = 6.0
    float det = determinant(m);
    return det;
    // Should be 6.0
}

// run: test_mat2_determinant_simple() ~= 6.0

float test_mat3_determinant() {
    mat3 m = mat3(1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0);
    // Diagonal matrix: det = 1.0 * 2.0 * 3.0 = 6.0
    float det = determinant(m);
    return det;
    // Should be 6.0
}

// run: test_mat3_determinant() ~= 6.0

float test_mat3_determinant_identity() {
    mat3 m = mat3(1.0);  // Identity matrix
    float det = determinant(m);
    return det;
    // Should be 1.0
}

// run: test_mat3_determinant_identity() ~= 1.0

float test_mat4_determinant_identity() {
    mat4 m = mat4(1.0);  // Identity matrix
    float det = determinant(m);
    return det;
    // Should be 1.0
}

// run: test_mat4_determinant_identity() ~= 1.0

