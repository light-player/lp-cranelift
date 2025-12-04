// test compile
// test run

float main() {
    float sum = 0.0;
    for (int i = 0; i < 3; i = i + 1) {
        sum = sum + 1.5;
    }
    return sum;
}

// CHECK: f32const
// CHECK: fadd
// run: == 4.5

