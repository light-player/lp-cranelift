// test compile
// test run

vec3 main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    return v.xyz;  // Should return vec3(1.0, 2.0, 3.0)
}

// run: == vec3(1.0, 2.0, 3.0)




