// test compile

vec3 main() {
    vec3 edge = vec3(5.0, 5.0, 5.0);
    vec3 x = vec3(3.0, 5.0, 7.0);
    return step(edge, x);  // (0.0, 1.0, 1.0)
}

// CHECK: fcmp
// CHECK: select

