// test compile
// test fixed32

float main() {
    return 3.14159;
}

// CHECK: iconst.i64
// CHECK-NOT: f32const

