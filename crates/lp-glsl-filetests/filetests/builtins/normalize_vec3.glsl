// test compile

vec3 main() {
    vec3 v = vec3(3.0, 0.0, 4.0);  // length = 5.0
    return normalize(v);  // (0.6, 0.0, 0.8)
}

// CHECK: sqrt
// CHECK: fdiv

