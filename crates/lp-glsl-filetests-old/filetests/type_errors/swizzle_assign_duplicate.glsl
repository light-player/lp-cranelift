// test error

void main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xx = vec2(5.0, 6.0);  // ERROR: 'x' used twice
}

// EXPECT_ERROR_CODE: E0113
// EXPECT_ERROR: swizzle `xx` contains duplicate components (illegal in assignment)
// EXPECT_LOCATION: 4:7



