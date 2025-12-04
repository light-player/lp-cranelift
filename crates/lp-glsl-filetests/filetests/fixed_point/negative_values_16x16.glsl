// test compile
// test fixed16

float main() {
    float a = -5.5;
    float b = 2.25;
    return a + b;
}

// CHECK: iconst.i32 -360448
// CHECK: iconst.i32 147456
// CHECK: iadd

