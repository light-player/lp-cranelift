// test run
// target riscv32.fixed32

// ============================================================================
// Compound assignment: +=, -=, *=, /=
// ============================================================================

float test_mat2_add_assign() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 m2 = mat2(10.0, 20.0, 30.0, 40.0);
    m += m2;
    // Component-wise addition in place
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 11.0 + 22.0 + 33.0 + 44.0 = 110.0
}

// run: test_mat2_add_assign() ~= 110.0

float test_mat2_sub_assign() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    mat2 m2 = mat2(1.0, 2.0, 3.0, 4.0);
    m -= m2;
    // Component-wise subtraction in place
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 9.0 + 18.0 + 27.0 + 36.0 = 90.0
}

// run: test_mat2_sub_assign() ~= 90.0

float test_mat2_mul_assign_scalar() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    float s = 2.0;
    m *= s;
    // Component-wise multiplication in place
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 2.0 + 4.0 + 6.0 + 8.0 = 20.0
}

// run: test_mat2_mul_assign_scalar() ~= 20.0

float test_mat2_div_assign_scalar() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    float s = 2.0;
    m /= s;
    // Component-wise division in place
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 5.0 + 10.0 + 15.0 + 20.0 = 50.0
}

// run: test_mat2_div_assign_scalar() ~= 50.0

float test_mat3_add_assign() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 m2 = mat3(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0);
    m += m2;
    // Sum diagonal
    return m[0][0] + m[1][1] + m[2][2];
    // Should be 11.0 + 55.0 + 99.0 = 165.0
}

// run: test_mat3_add_assign() ~= 165.0

float test_mat4_mul_assign_scalar() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    float s = 2.0;
    m *= s;
    // Sum first row
    return m[0][0] + m[0][1] + m[0][2] + m[0][3];
    // Should be 2.0 + 6.0 + 18.0 + 26.0 = 52.0
}

// run: test_mat4_mul_assign_scalar() ~= 52.0


