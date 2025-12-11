// test error

int main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    v.x = vec2(4.0, 5.0);  // ERROR: assigning vector to scalar component
    return 1;
}

// EXPECT_ERROR_CODE: E0400
// EXPECT_ERROR: swizzle assignment size mismatch: 1 components on LHS, 2 on RHS
// EXPECT_LOCATION: 4:11



