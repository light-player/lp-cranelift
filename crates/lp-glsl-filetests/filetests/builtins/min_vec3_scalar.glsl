// test compile

vec3 main() {
    vec3 v = vec3(5.0, 2.0, 7.0);
    return min(v, 4.0);  // (4.0, 2.0, 4.0)
}

// CHECK: fmin

