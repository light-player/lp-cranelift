// test compile
// test run

float main() {
    int x = 5;
    float y = 2.5;
    return x + y;  // x implicitly converted to float
}

// CHECK: iconst.i32 5
// CHECK: fcvt_from_sint
// CHECK: f32const
// CHECK: fadd
// run: == 7.5

