// test compile
// test run

vec2 main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    return a.xy + b.yz;  // vec2(1.0, 2.0) + vec2(5.0, 6.0) = vec2(6.0, 8.0)
}

// run: == vec2(6.0, 8.0)






