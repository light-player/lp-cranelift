// test compile
// test fixed32

float main() {
    float a = 10.0;
    float b = 4.0;
    return a / b;
}

// CHECK: sextend.i128
// CHECK: iconst.i64 32
// CHECK: ishl
// CHECK: sdiv
// CHECK: ireduce.i64

