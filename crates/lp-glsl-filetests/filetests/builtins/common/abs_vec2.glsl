// test compile
// test run

vec2 main() {
    return abs(vec2(-1.5, 2.3));  // Should return (1.5, 2.3)
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.800000p0
//     v2 = fneg v1  ; v1 = 0x1.800000p0
//     v3 = f32const 0x1.266666p1
//     v4 = fabs v2
//     v5 = fabs v3  ; v3 = 0x1.266666p1
//     store notrap aligned v4, v0
//     store notrap aligned v5, v0+4
//     return
//
// block1:
//     v6 = f32const 0.0
//     store notrap aligned v6, v0  ; v6 = 0.0
//     v7 = f32const 0.0
//     store notrap aligned v7, v0+4  ; v7 = 0.0
//     return
// }
// run: ≈ vec2(1.5, 2.3) (tolerance: 0.01)
