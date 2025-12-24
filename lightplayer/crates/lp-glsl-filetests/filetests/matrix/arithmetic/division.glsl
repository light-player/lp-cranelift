// test run
// target riscv32.fixed32

// ============================================================================
// Matrix division: m / scalar (component-wise)
// ============================================================================

float test_mat2_division() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    float s = 2.0;
    mat2 result = m / s;
    // Component-wise division
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_mat2_division() ~= 50.0

float test_mat3_division() {
    mat3 m = mat3(6.0, 12.0, 18.0, 24.0, 30.0, 36.0, 42.0, 48.0, 54.0);
    float s = 3.0;
    mat3 result = m / s;
    // Sum diagonal
    return result[0][0] + result[1][1] + result[2][2];
    // Should be 2.0 + 10.0 + 18.0 = 30.0
}

// run: test_mat3_division() ~= 30.0

float test_mat4_division() {
    mat4 m = mat4(
        20.0, 40.0, 60.0, 80.0,
        100.0, 120.0, 140.0, 160.0,
        180.0, 200.0, 220.0, 240.0,
        260.0, 280.0, 300.0, 320.0
    );
    float s = 4.0;
    mat4 result = m / s;
    // Sum column 0 (all rows of column 0)
    // Column 0: [5.0, 25.0, 45.0, 65.0] (from [20,100,180,260] / 4.0)
    return result[0][0] + result[0][1] + result[0][2] + result[0][3];
    // Should be 5.0 + 25.0 + 45.0 + 65.0 = 140.0
}

// run: test_mat4_division() ~= 140.0

