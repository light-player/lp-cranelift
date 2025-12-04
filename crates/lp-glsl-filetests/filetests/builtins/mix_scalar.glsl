// test compile

float main() {
    return mix(0.0, 10.0, 0.5);  // Should return 5.0
}

// CHECK: fmul
// CHECK: fadd

