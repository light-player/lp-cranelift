// test compile
// test run

float main() {
    return min(5.0, 3.0);  // 3.0
}

// CHECK: fmin
// run: ~= 3.0 (tolerance: 0.01)

