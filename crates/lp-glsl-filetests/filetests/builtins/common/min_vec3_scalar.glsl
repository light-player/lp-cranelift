// test compile

vec3 main() {
    vec3 v = vec3(5.0, 2.0, 7.0);
    return min(v, 4.0);  // (4.0, 2.0, 4.0)
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.400000p2
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.c00000p2
//     v4 = f32const 0x1.000000p2
//     v5 = fmin v1, v4  ; v1 = 0x1.400000p2, v4 = 0x1.000000p2
//     v6 = fmin v2, v4  ; v2 = 0x1.000000p1, v4 = 0x1.000000p2
//     v7 = fmin v3, v4  ; v3 = 0x1.c00000p2, v4 = 0x1.000000p2
//     store notrap aligned v5, v0
//     store notrap aligned v6, v0+4
//     store notrap aligned v7, v0+8
//     return
//
// block1:
//     v8 = f32const 0.0
//     store notrap aligned v8, v0  ; v8 = 0.0
//     v9 = f32const 0.0
//     store notrap aligned v9, v0+4  ; v9 = 0.0
//     v10 = f32const 0.0
//     store notrap aligned v10, v0+8  ; v10 = 0.0
//     return
// }
