// test compile
// test run

int main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return 1;  // Just test construction succeeds
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = iconst.i32 1
//     return v3  ; v3 = 1
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
// run: == 1
