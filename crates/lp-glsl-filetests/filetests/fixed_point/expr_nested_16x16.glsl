// test compile
// test fixed16

float main() {
    float x = 1.5;
    float y = 2.5;
    return ((x * 2.0) + (y / 2.0)) - 0.5;
}

// CHECK: imul
// CHECK: sdiv
// CHECK: iadd
// CHECK: isub

