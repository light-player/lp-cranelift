// test compile
// test run

vec3 main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    return mix(a, b, vec3(0.5, 0.5, 0.5));  // (5.0, 10.0, 15.0)
}

// run: == vec3(5.0, 10.0, 15.0)

