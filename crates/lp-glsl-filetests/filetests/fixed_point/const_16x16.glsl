// test compile
// test fixed16

float main() {
    return 3.14159;
}

// CHECK: iconst.i32 205887
// CHECK-NOT: f32const

