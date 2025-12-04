// test compile
// test fixed16

float main() {
    float a = 0.0000152587890625;
    return a + a;
}

// CHECK: iconst.i32 1
// CHECK: iadd

