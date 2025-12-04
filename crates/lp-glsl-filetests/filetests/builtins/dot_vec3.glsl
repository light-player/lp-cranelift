// test compile
// test run

float main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    return dot(a, b);  // 1*4 + 2*5 + 3*6 = 32.0
}

// CHECK: fmul
// CHECK: fadd
// run: == 32.0

