// test compile

vec2 main() {
    vec2 a = vec2(1.0, 5.0);
    vec2 b = vec2(3.0, 2.0);
    return max(a, b);  // (3.0, 5.0)
}

// CHECK: fmax

