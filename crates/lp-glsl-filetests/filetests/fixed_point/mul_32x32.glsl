// test compile
// test fixed32

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// CHECK: sextend.i128
// CHECK: imul
// CHECK: iconst.i64 32
// CHECK: sshr
// CHECK: ireduce.i64

