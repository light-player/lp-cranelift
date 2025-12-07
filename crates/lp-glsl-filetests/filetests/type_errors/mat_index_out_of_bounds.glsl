// test error

vec2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return m[2];  // ERROR: index 2 out of bounds (max 1)
}

// EXPECT_ERROR_CODE: E0400
// EXPECT_ERROR: matrix column index 2 out of bounds (max 1)
// EXPECT_LOCATION: 5

