// test compile

float main() {
    return mod(7.0, 3.0);  // Should return 1.0
}

// CHECK: floor
// CHECK: fmul
// CHECK: fsub

