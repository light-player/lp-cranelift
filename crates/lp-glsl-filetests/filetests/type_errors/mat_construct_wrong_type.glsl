// test error

mat2 main() {
    return mat2(true, false, true, false);  // ERROR: bool cannot be used in matrix constructor
}

// EXPECT_ERROR_CODE: E0103
// EXPECT_ERROR: cannot construct `mat2` from `Bool`
// EXPECT_LOCATION: 4




