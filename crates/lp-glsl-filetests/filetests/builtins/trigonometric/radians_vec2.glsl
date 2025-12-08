// test compile
// test run

vec2 main() {
    vec2 deg = vec2(90.0, 45.0);
    return radians(deg);
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.680000p6
//     v2 = f32const 0x1.680000p5
//     v3 = f32const 0x1.1df46ap-6
//     v4 = fmul v1, v3  ; v1 = 0x1.680000p6, v3 = 0x1.1df46ap-6
//     v5 = fmul v2, v3  ; v2 = 0x1.680000p5, v3 = 0x1.1df46ap-6
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
// run: ≈ vec2(1.5708, 0.7854) (tolerance: 0.01)
