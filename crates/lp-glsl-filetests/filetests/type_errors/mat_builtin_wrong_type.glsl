// test error

float main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return determinant(v);  // ERROR: determinant requires matrix, got vector
}

// EXPECT_ERROR_CODE: E0104
// EXPECT_ERROR: determinant() requires a matrix
// EXPECT_LOCATION: 5



