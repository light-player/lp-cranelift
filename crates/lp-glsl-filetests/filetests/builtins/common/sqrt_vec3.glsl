// test compile
// test run

vec3 main() {
    return sqrt(vec3(4.0, 9.0, 16.0));  // Should return (2.0, 3.0, 4.0)
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.000000p2
//     v2 = f32const 0x1.200000p3
//     v3 = f32const 0x1.000000p4
//     v4 = sqrt v1  ; v1 = 0x1.000000p2
//     v5 = sqrt v2  ; v2 = 0x1.200000p3
//     v6 = sqrt v3  ; v3 = 0x1.000000p4
//     store notrap aligned v4, v0
//     store notrap aligned v5, v0+4
//     store notrap aligned v6, v0+8
//     return
//
// block1:
//     v7 = f32const 0.0
//     store notrap aligned v7, v0  ; v7 = 0.0
//     v8 = f32const 0.0
//     store notrap aligned v8, v0+4  ; v8 = 0.0
//     v9 = f32const 0.0
//     store notrap aligned v9, v0+8  ; v9 = 0.0
//     return
// }
// run: ≈ vec3(2, 3, 4) (tolerance: 0.01)
