// test compile
// test run

vec4 main() {
    vec2 v = vec2(1.0, 2.0);
    return v.xxyy;  // Should return vec4(1.0, 1.0, 2.0, 2.0)
}

// run: == vec4(1.0, 1.0, 2.0, 2.0)




