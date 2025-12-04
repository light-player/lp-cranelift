// test compile

float main() {
    return fract(3.75);  // Should return 0.75
}

// CHECK: floor
// CHECK: fsub

