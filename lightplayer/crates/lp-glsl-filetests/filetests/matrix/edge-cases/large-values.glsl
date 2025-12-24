// test run
// target riscv32.fixed32

// ============================================================================
// Large and small value tests (precision limits)
// ============================================================================

float test_mat2_large_values() {
    mat2 m = mat2(1000.0, 2000.0, 3000.0, 4000.0);
    // Test with large values
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1000.0 + 2000.0 + 3000.0 + 4000.0 = 10000.0
}

// run: test_mat2_large_values() ~= 10000.0

float test_mat2_small_values() {
    mat2 m = mat2(0.001, 0.002, 0.003, 0.004);
    // Test with small values
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 0.001 + 0.002 + 0.003 + 0.004 = 0.01
}

// run: test_mat2_small_values() ~= 0.01

float test_mat2_mixed_precision() {
    mat2 m1 = mat2(1000.0, 2000.0, 3000.0, 4000.0);
    mat2 m2 = mat2(0.001, 0.002, 0.003, 0.004);
    mat2 result = m1 + m2;
    // Large + small values
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 1000.001 + 2000.002 + 3000.003 + 4000.004 = 10000.01
}

// run: test_mat2_mixed_precision() ~= 10000.01

float test_mat3_large_multiplication() {
    mat3 m = mat3(100.0, 200.0, 300.0, 400.0, 500.0, 600.0, 700.0, 800.0, 900.0);
    float s = 10.0;
    mat3 result = m * s;
    // Large values * scalar
    return result[0][0] + result[1][1] + result[2][2];
    // Should be 1000.0 + 5000.0 + 9000.0 = 15000.0
}

// run: test_mat3_large_multiplication() ~= 15000.0

float test_mat4_precision() {
    mat4 m = mat4(
        0.0001, 0.0002, 0.0003, 0.0004,
        0.0005, 0.0006, 0.0007, 0.0008,
        0.0009, 0.0010, 0.0011, 0.0012,
        0.0013, 0.0014, 0.0015, 0.0016
    );
    // Sum column 0 (all rows of column 0)
    // Column 0: [0.0001, 0.0005, 0.0009, 0.0013]
    return m[0][0] + m[0][1] + m[0][2] + m[0][3];
    // Should be 0.0001 + 0.0005 + 0.0009 + 0.0013 = 0.0028
}

// run: test_mat4_precision() ~= 0.0010070801

