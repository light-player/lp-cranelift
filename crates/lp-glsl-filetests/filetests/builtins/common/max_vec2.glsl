// test compile

vec2 main() {
    vec2 a = vec2(1.0, 5.0);
    vec2 b = vec2(3.0, 2.0);
    return max(a, b);  // (3.0, 5.0)
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.400000p2
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p1
//     v5 = fmax v1, v3  ; v1 = 0x1.000000p0, v3 = 0x1.800000p1
//     v6 = fmax v2, v4  ; v2 = 0x1.400000p2, v4 = 0x1.000000p1
//     store notrap aligned v5, v0
//     store notrap aligned v6, v0+4
//     return
//
// block1:
//     v7 = f32const 0.0
//     store notrap aligned v7, v0  ; v7 = 0.0
//     v8 = f32const 0.0
//     store notrap aligned v8, v0+4  ; v8 = 0.0
//     return
// }
