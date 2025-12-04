// test compile
// test run

float main() {
    return clamp(7.0, 2.0, 5.0);  // 5.0
}

// CHECK: fmin
// CHECK: fmax
// run: == 5.0

