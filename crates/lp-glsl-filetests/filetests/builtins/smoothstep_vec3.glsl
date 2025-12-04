// test compile

vec3 main() {
    vec3 edge0 = vec3(0.0, 0.0, 0.0);
    vec3 edge1 = vec3(10.0, 10.0, 10.0);
    vec3 x = vec3(5.0, 2.5, 7.5);
    return smoothstep(edge0, edge1, x);
}

// CHECK: fmin
// CHECK: fmax

