// test compile
// test run

float main() {
    vec2 v = vec2(3.0, 4.0);
    return length(v);  // sqrt(9 + 16) = 5.0
}

// CHECK: fmul
// CHECK: fadd
// CHECK: sqrt
// run: == 5.0

