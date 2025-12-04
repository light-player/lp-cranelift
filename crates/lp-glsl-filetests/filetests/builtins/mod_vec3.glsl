// test compile

vec3 main() {
    return mod(vec3(7.0, 8.0, 9.0), vec3(3.0, 3.0, 4.0));  // (1.0, 2.0, 1.0)
}

// CHECK: floor

