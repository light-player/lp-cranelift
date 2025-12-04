// test compile
// test run

vec3 main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    v.xy = vec2(5.0, 6.0);
    return v;  // Should return vec3(5.0, 6.0, 3.0)
}

// run: == vec3(5.0, 6.0, 3.0)

