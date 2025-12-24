// test run
// target riscv32.fixed32

// ============================================================================
// Matrix constructor from multiple scalars (column-major order)
// mat2(1.0, 2.0, 3.0, 4.0) fills in column-major order:
//   Column 0: [1.0, 2.0]
//   Column 1: [3.0, 4.0]
//   Matrix: [1.0, 3.0; 2.0, 4.0]
// ============================================================================

float test_mat2_from_scalars() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    // Column-major: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
    // m[0][0]=1.0 (col 0, row 0), m[0][1]=2.0 (col 0, row 1)
    // m[1][0]=3.0 (col 1, row 0), m[1][1]=4.0 (col 1, row 1)
    return m[0][0] + m[1][0] + m[0][1] + m[1][1];
    // Should be 1.0 + 3.0 + 2.0 + 4.0 = 10.0
}

// run: test_mat2_from_scalars() ~= 10.0

float test_mat2_from_scalars_verify_order() {
    mat2 m = mat2(10.0, 20.0, 30.0, 40.0);
    // Verify column-major order
    // Column 0: [10.0, 20.0] → m[0][0]=10.0, m[0][1]=20.0
    // Column 1: [30.0, 40.0] → m[1][0]=30.0, m[1][1]=40.0
    float col0 = m[0][0] + m[0][1];  // 10.0 + 20.0 = 30.0
    float col1 = m[1][0] + m[1][1];  // 30.0 + 40.0 = 70.0
    return col0 + col1;
    // Should be 30.0 + 70.0 = 100.0
}

// run: test_mat2_from_scalars_verify_order() ~= 100.0

float test_mat3_from_scalars() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    // Column-major order:
    // Column 0: [1.0, 2.0, 3.0] → m[0][0]=1.0, m[0][1]=2.0, m[0][2]=3.0
    // Column 1: [4.0, 5.0, 6.0] → m[1][0]=4.0, m[1][1]=5.0, m[1][2]=6.0
    // Column 2: [7.0, 8.0, 9.0] → m[2][0]=7.0, m[2][1]=8.0, m[2][2]=9.0
    // Verify first element of each column (row 0)
    return m[0][0] + m[1][0] + m[2][0];
    // Should be 1.0 + 4.0 + 7.0 = 12.0
}

// run: test_mat3_from_scalars() ~= 12.0

float test_mat3_from_scalars_all() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    // Sum all elements (column-major order)
    return m[0][0] + m[0][1] + m[0][2] +  // Column 0: 1+2+3 = 6
           m[1][0] + m[1][1] + m[1][2] +  // Column 1: 4+5+6 = 15
           m[2][0] + m[2][1] + m[2][2];   // Column 2: 7+8+9 = 24
    // Should be 6 + 15 + 24 = 45.0
}

// run: test_mat3_from_scalars_all() ~= 45.0

float test_mat4_from_scalars() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Column-major order:
    // Column 0: [1.0, 5.0, 9.0, 13.0] → m[0][0]=1.0, m[0][1]=5.0, m[0][2]=9.0, m[0][3]=13.0
    // Column 1: [2.0, 6.0, 10.0, 14.0] → m[1][0]=2.0, m[1][1]=6.0, m[1][2]=10.0, m[1][3]=14.0
    // Column 2: [3.0, 7.0, 11.0, 15.0] → m[2][0]=3.0, m[2][1]=7.0, m[2][2]=11.0, m[2][3]=15.0
    // Column 3: [4.0, 8.0, 12.0, 16.0] → m[3][0]=4.0, m[3][1]=8.0, m[3][2]=12.0, m[3][3]=16.0
    // Sum column 0 (all rows of column 0)
    return m[0][0] + m[0][1] + m[0][2] + m[0][3];
    // Should be 1.0 + 5.0 + 9.0 + 13.0 = 28.0
}

// run: test_mat4_from_scalars() ~= 28.0

float test_mat4_from_scalars_column_verify() {
    mat4 m = mat4(
        1.0, 2.0, 3.0, 4.0,
        5.0, 6.0, 7.0, 8.0,
        9.0, 10.0, 11.0, 12.0,
        13.0, 14.0, 15.0, 16.0
    );
    // Verify column 1 (second column) elements: [2.0, 6.0, 10.0, 14.0]
    return m[1][0] + m[1][1] + m[1][2] + m[1][3];
    // Should be 2.0 + 6.0 + 10.0 + 14.0 = 32.0
}

// run: test_mat4_from_scalars_column_verify() ~= 32.0

