// test error

vec3 main() {
    mat3 m = mat3(1.0);
    vec2 v = vec2(1.0, 2.0);
    return m * v;  // ERROR: mat3 requires vec3, got vec2
}

// EXPECT_ERROR_CODE: E0106
// EXPECT_ERROR: matrix × vector dimension mismatch
// EXPECT_LOCATION: 6

