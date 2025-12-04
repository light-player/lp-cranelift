// test compile
// test run

float square(float x) {
    return x * x;
}

float main() {
    return square(5.0);  // 25.0
}

// CHECK: fmul
// run: ~= 25.0 (tolerance: 0.01)

