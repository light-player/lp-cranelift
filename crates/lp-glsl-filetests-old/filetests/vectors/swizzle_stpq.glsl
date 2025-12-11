// test compile
// test run

vec2 main() {
    vec4 tc = vec4(0.1, 0.2, 0.3, 0.4);
    return tc.st;  // Should return vec2(0.1, 0.2)
}

// run: == vec2(0.1, 0.2)






