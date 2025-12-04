// test error

vec2 main() {
    vec2 a = vec2(1.0, 2.0);
    vec2 b = vec2(3.0, 4.0);
    return cross(a, b);  // ERROR: cross requires vec3
}

// EXPECT_ERROR: cross() requires vec3 arguments

