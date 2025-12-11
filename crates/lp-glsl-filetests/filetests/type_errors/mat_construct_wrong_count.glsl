// test error

mat2 main() {
    return mat2(1.0, 2.0, 3.0);  // ERROR: wrong number of arguments
}

// EXPECT_ERROR_CODE: E0115
// EXPECT_ERROR: `mat2` constructor has wrong number of arguments
// EXPECT_LOCATION: 4




