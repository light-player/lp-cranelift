// test compile
// test run

vec3 main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    return mix(a, b, 0.25);  // (2.5, 5.0, 7.5)
}

// run: == vec3(2.5, 5.0, 7.5)

