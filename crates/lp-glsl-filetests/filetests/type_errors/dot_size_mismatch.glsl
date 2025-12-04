// test error

float main() {
    vec2 a = vec2(1.0, 2.0);
    vec3 b = vec3(3.0, 4.0, 5.0);
    return dot(a, b);  // ERROR: size mismatch
}

// EXPECT_ERROR: dot() requires matching vector sizes

