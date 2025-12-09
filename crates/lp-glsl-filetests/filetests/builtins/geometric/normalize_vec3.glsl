// test compile

// target riscv32
vec3 main() {
    vec3 v = vec3(3.0, 0.0, 4.0);  // length = 5.0
    return normalize(v);  // (0.6, 0.0, 0.8)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.800000p1
//     v2 = f32const 0.0
//     v3 = f32const 0x1.000000p2
//     v4 = fmul v1, v1  ; v1 = 0x1.800000p1, v1 = 0x1.800000p1
//     v5 = fmul v2, v2  ; v2 = 0.0, v2 = 0.0
//     v6 = fadd v4, v5
//     v7 = fmul v3, v3  ; v3 = 0x1.000000p2, v3 = 0x1.000000p2
//     v8 = fadd v6, v7
//     v9 = sqrt v8
//     v10 = fdiv v1, v9  ; v1 = 0x1.800000p1
//     v11 = fdiv v2, v9  ; v2 = 0.0
//     v12 = fdiv v3, v9  ; v3 = 0x1.000000p2
//     store notrap aligned v10, v0
//     store notrap aligned v11, v0+4
//     store notrap aligned v12, v0+8
//     return
//
// block1:
//     v13 = f32const 0.0
//     store notrap aligned v13, v0  ; v13 = 0.0
//     v14 = f32const 0.0
//     store notrap aligned v14, v0+4  ; v14 = 0.0
//     v15 = f32const 0.0
//     store notrap aligned v15, v0+8  ; v15 = 0.0
//     return
// }
