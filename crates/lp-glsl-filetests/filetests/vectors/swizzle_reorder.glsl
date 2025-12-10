// test compile
// test run

vec3 main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return v.zyx;  // Should return vec3(3.0, 2.0, 1.0)
}

// run: == vec3(3.0, 2.0, 1.0)




