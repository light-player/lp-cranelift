// test run
// target riscv32.fixed32

// ============================================================================
// Simple matrix assignment: m1 = m2
// ============================================================================

float test_mat2_assignment() {
    mat2 m1 = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 m2 = mat2(10.0, 20.0, 30.0, 40.0);
    m1 = m2;
    // After assignment, m1 should equal m2
    return m1[0][0] + m1[1][0] + m1[0][1] + m1[1][1];
    // Should be 10.0 + 20.0 + 30.0 + 40.0 = 100.0
}

// run: test_mat2_assignment() ~= 100.0

float test_mat2_assignment_independence() {
    mat2 m1 = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 m2 = mat2(10.0, 20.0, 30.0, 40.0);
    m1 = m2;
    m2[0][0] = 100.0;  // Modify m2
    // m1 should be unchanged (independent copy)
    return m1[0][0];
    // Should be 10.0 (not 100.0)
}

// run: test_mat2_assignment_independence() ~= 10.0

float test_mat3_assignment() {
    mat3 m1 = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    mat3 m2 = mat3(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0);
    m1 = m2;
    // Sum diagonal
    return m1[0][0] + m1[1][1] + m1[2][2];
    // Should be 10.0 + 50.0 + 90.0 = 150.0
}

// run: test_mat3_assignment() ~= 150.0

float test_mat4_assignment() {
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
    m1 = m2;
    // Sum column 0 (all rows of column 0)
    return m1[0][0] + m1[0][1] + m1[0][2] + m1[0][3];
    // Should be 10.0 + 50.0 + 90.0 + 130.0 = 280.0
}

// run: test_mat4_assignment() ~= 100.0

