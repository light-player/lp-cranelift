// test run
// target riscv32.fixed32

// ============================================================================
// Singular matrix tests (non-invertible, determinant = 0)
// ============================================================================

float test_mat2_singular_determinant() {
    // Singular matrix: two rows/columns are linearly dependent
    mat2 m = mat2(1.0, 2.0, 2.0, 4.0);
    // Column 1 = 2 * Column 0, so det = 0
    float det = determinant(m);
    return det;
    // Should be 0.0 (or very close to 0.0)
}

// run: test_mat2_singular_determinant() ~= 0.0

float test_mat2_singular_zero_column() {
    // Matrix with zero column
    mat2 m = mat2(1.0, 2.0, 0.0, 0.0);
    float det = determinant(m);
    return det;
    // Should be 0.0
}

// run: test_mat2_singular_zero_column() ~= 0.0

float test_mat3_singular() {
    // Singular matrix: one column is zero
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 0.0, 0.0, 0.0);
    float det = determinant(m);
    return det;
    // Should be 0.0
}

// run: test_mat3_singular() ~= 0.0

float test_mat4_singular() {
    // Singular matrix: two identical columns
    mat4 m = mat4(
        1.0, 2.0, 1.0, 4.0,
        5.0, 6.0, 5.0, 8.0,
        9.0, 10.0, 9.0, 12.0,
        13.0, 14.0, 13.0, 16.0
    );
    // Columns 0 and 2 are identical, so det = 0
    float det = determinant(m);
    return det;
    // Should be 0.0 (or very close to 0.0)
}

// run: test_mat4_singular() ~= 0.0


