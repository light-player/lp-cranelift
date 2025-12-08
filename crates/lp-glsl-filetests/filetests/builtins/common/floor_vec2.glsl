// test compile
// test run

vec2 main() {
    return floor(vec2(3.7, -2.3));  // Should return (3.0, -3.0)
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.d9999ap1
//     v2 = f32const 0x1.266666p1
//     v3 = fneg v2  ; v2 = 0x1.266666p1
//     v4 = floor v1  ; v1 = 0x1.d9999ap1
//     v5 = floor v3
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
// run: ≈ vec2(3.0, -3.0) (tolerance: 0.01)
