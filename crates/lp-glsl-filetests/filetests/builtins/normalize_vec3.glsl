// test compile

vec3 main() {
    vec3 v = vec3(3.0, 0.0, 4.0);  // length = 5.0
    return normalize(v);  // (0.6, 0.0, 0.8)
}

// function u0:0() -> f32, f32, f32 fast {
// block0:
//     v0 = f32const 0x1.800000p1
//     v1 = f32const 0.0
//     v2 = f32const 0x1.000000p2
//     v3 = fmul v0, v0  ; v0 = 0x1.800000p1, v0 = 0x1.800000p1
//     v4 = fmul v1, v1  ; v1 = 0.0, v1 = 0.0
//     v5 = fadd v3, v4
//     v6 = fmul v2, v2  ; v2 = 0x1.000000p2, v2 = 0x1.000000p2
//     v7 = fadd v5, v6
//     v8 = sqrt v7
//     v9 = fdiv v0, v8  ; v0 = 0x1.800000p1
//     v10 = fdiv v1, v8  ; v1 = 0.0
//     v11 = fdiv v2, v8  ; v2 = 0x1.000000p2
//     return v9
//
// block1:
//     v12 = f32const 0.0
//     v13 = f32const 0.0
//     v14 = f32const 0.0
//     return v12, v13, v14  ; v12 = 0.0, v13 = 0.0, v14 = 0.0
// }
