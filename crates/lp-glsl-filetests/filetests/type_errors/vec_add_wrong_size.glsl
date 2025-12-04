// test error

int main() {
    vec2 a = vec2(1.0, 2.0);
    vec3 b = vec3(3.0, 4.0, 5.0);
    vec3 c = a + b;  // ERROR: size mismatch
    return 1;
}

// EXPECT_ERROR: requires matching types

