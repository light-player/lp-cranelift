// test error

mat3 main() {
    mat2 a = mat2(1.0);
    mat3 b = mat3(1.0);
    return a * b;  // ERROR: incompatible matrix dimensions
}

// EXPECT_ERROR_CODE: E0106
// EXPECT_ERROR: matrix × matrix dimension mismatch
// EXPECT_LOCATION: 6

