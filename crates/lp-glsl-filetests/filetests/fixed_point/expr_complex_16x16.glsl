// test compile
// test fixed16

float main() {
    float a = 2.0;
    float b = 3.0;
    float c = 4.0;
    return (a + b) * c - 1.5;
}

// CHECK: iadd
// CHECK: imul
// CHECK: isub
// CHECK-NOT: fadd
// CHECK-NOT: fmul
// CHECK-NOT: fsub

