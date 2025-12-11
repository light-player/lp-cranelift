// test error

float main() {
    vec2 a = vec2(1.0, 2.0);
    vec3 b = vec3(3.0, 4.0, 5.0);
    return dot(a, b);  // ERROR: size mismatch
}

// EXPECT_ERROR_CODE: E0114
// EXPECT_ERROR: No matching overload for dot([Vec2, Vec3])
// EXPECT_LOCATION: 5:12



