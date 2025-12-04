// test compile
// test run

float main() {
    float x = 10;  // int 10 → float conversion
    return x;
}

// CHECK: iconst.i32 10
// CHECK: fcvt_from_sint
// run: == 10.0

