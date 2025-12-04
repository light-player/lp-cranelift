// test compile

vec3 main() {
    return smoothstep(0.0, 10.0, vec3(0.0, 5.0, 10.0));
}

// CHECK: fmin
// CHECK: fmax

