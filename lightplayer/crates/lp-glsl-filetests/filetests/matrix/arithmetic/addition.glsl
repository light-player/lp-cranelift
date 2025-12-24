// test run
// target riscv32.fixed32

// ============================================================================
// Matrix addition: m1 + m2 (component-wise)
// ============================================================================

float test_mat2_addition() {
    mat2 m1 = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 m2 = mat2(10.0, 20.0, 30.0, 40.0);
    mat2 result = m1 + m2;
    // Component-wise addition
    // [1.0, 3.0; 2.0, 4.0] + [10.0, 30.0; 20.0, 40.0] = [11.0, 33.0; 22.0, 44.0]
    return result[0][0] + result[1][0] + result[0][1] + result[1][1];
    // Should be 11.0 + 22.0 + 33.0 + 44.0 = 110.0
}

// run: test_mat2_addition() ~= 110.0

float test_mat3_addition() {
    mat3 m1 = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 m2 = mat3(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0);
    mat3 result = m1 + m2;
    // Sum diagonal elements
    return result[0][0] + result[1][1] + result[2][2];
    // Should be 11.0 + 55.0 + 99.0 = 165.0
}

// run: test_mat3_addition() ~= 165.0

float test_mat4_addition() {
    mat4 m1 = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    mat4 m2 = mat4(
        10.0, 20.0, 30.0, 40.0,
        50.0, 60.0, 70.0, 80.0,
        90.0, 100.0, 110.0, 120.0,
        130.0, 140.0, 150.0, 160.0
    );
    mat4 result = m1 + m2;
    // Sum column 0 (all rows of column 0)
    // Column 0: [11.0, 55.0, 99.0, 143.0] (from [1,5,9,13] + [10,50,90,130])
    return result[0][0] + result[0][1] + result[0][2] + result[0][3];
    // Should be 11.0 + 55.0 + 99.0 + 143.0 = 308.0
}

// run: test_mat4_addition() ~= 308.0

