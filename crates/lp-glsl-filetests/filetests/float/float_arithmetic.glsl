// test compile
// test run

float main() {
    float a = 2.5;
    float b = 1.5;
    return a + b;
}

// CHECK: f32const
// CHECK: fadd
// run: == 4.0

