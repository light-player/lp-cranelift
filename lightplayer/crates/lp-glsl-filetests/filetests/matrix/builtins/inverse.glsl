// test run
// target riscv32.fixed32

// ============================================================================
// Matrix inverse: inverse(m)
// For invertible matrices: m * inverse(m) ≈ identity
// ============================================================================

float test_mat2_inverse_identity() {
    mat2 m = mat2(1.0);  // Identity matrix
    mat2 inv = inverse(m);
    // Inverse of identity is identity
    return inv[0][0] + inv[1][1];
    // Should be 1.0 + 1.0 = 2.0
}

// run: test_mat2_inverse_identity() ~= 2.0

float test_mat2_inverse_simple() {
    mat2 m = mat2(2.0, 0.0, 0.0, 4.0);
    // Diagonal matrix: inverse is [0.5, 0.0; 0.0, 0.25]
    mat2 inv = inverse(m);
    return inv[0][0] + inv[1][1];
    // Should be 0.5 + 0.25 = 0.75
}

// run: test_mat2_inverse_simple() ~= 0.75

float test_mat2_inverse_verify() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 inv = inverse(m);
    mat2 product = m * inv;
    // Product should be approximately identity
    // Check diagonal elements (should be close to 1.0)
    return product[0][0] + product[1][1];
    // Should be approximately 2.0 (1.0 + 1.0)
}

// run: test_mat2_inverse_verify() ~= 2.0

float test_mat3_inverse_identity() {
    mat3 m = mat3(1.0);  // Identity matrix
    mat3 inv = inverse(m);
    // Inverse of identity is identity
    return inv[0][0] + inv[1][1] + inv[2][2];
    // Should be 1.0 + 1.0 + 1.0 = 3.0
}

// run: test_mat3_inverse_identity() ~= 3.0

float test_mat4_inverse_identity() {
    mat4 m = mat4(1.0);  // Identity matrix
    mat4 inv = inverse(m);
    // Inverse of identity is identity
    return inv[0][0] + inv[1][1] + inv[2][2] + inv[3][3];
    // Should be 1.0 + 1.0 + 1.0 + 1.0 = 4.0
}

// run: test_mat4_inverse_identity() ~= 4.0

