// test run
// target riscv32.fixed32

// ============================================================================
// Matrix subtraction: m1 - m2 (component-wise)
// ============================================================================

float test_mat2_subtraction() {
    mat2 m1 = mat2(10.0, 20.0, 30.0, 40.0);
    mat2 m2 = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 result = m1 - m2;
    // Component-wise subtraction
    // [10.0, 30.0; 20.0, 40.0] - [1.0, 3.0; 2.0, 4.0] = [9.0, 27.0; 18.0, 36.0]
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 9.0 + 18.0 + 27.0 + 36.0 = 90.0
}

// run: test_mat2_subtraction() ~= 90.0

float test_mat3_subtraction() {
    mat3 m1 = mat3(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0);
    mat3 m2 = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 result = m1 - m2;
    // Sum diagonal
    return result[0][0] + result[1][1] + result[2][2];
    // Should be 9.0 + 45.0 + 81.0 = 135.0
}

// run: test_mat3_subtraction() ~= 135.0

float test_mat4_subtraction() {
    mat4 m1 = mat4(
        10.0, 20.0, 30.0, 40.0,
        50.0, 60.0, 70.0, 80.0,
        90.0, 100.0, 110.0, 120.0,
        130.0, 140.0, 150.0, 160.0
    );
    mat4 m2 = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    mat4 result = m1 - m2;
    // Sum column 0 (all rows of column 0)
    // Column 0: [9.0, 45.0, 81.0, 117.0] (from [10,50,90,130] - [1,5,9,13])
    return result[0][0] + result[0][1] + result[0][2] + result[0][3];
    // Should be 9.0 + 45.0 + 81.0 + 117.0 = 252.0
}

// run: test_mat4_subtraction() ~= 252.0

