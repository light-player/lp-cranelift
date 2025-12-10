// test compile
// test run

vec4 main() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    v.xyz = vec3(5.0, 6.0, 7.0);
    return v;  // Should return vec4(5.0, 6.0, 7.0, 4.0)
}

// run: == vec4(5.0, 6.0, 7.0, 4.0)




