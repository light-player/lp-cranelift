// test compile
// test run

float main() {
    int a = 3;
    float b = 2.0;
    float c = a * b;  // 3 → 3.0, then 3.0 * 2.0
    return c;
}

// CHECK: fcvt_from_sint
// CHECK: fmul
// run: ~= 6.0 (tolerance: 0.01)

