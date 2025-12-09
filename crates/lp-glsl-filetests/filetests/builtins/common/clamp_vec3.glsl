// test compile

// target riscv32
vec3 main() {
    vec3 v = vec3(-1.0, 0.5, 2.0);
    return clamp(v, 0.0, 1.0);  // (0.0, 0.5, 1.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.000000p0
//     v2 = fneg v1  ; v1 = 0x1.000000p0
//     v3 = f32const 0x1.000000p-1
//     v4 = f32const 0x1.000000p1
//     v5 = f32const 0.0
//     v6 = f32const 0x1.000000p0
//     v7 = fmax v2, v5  ; v5 = 0.0
//     v8 = fmax v3, v5  ; v3 = 0x1.000000p-1, v5 = 0.0
//     v9 = fmax v4, v5  ; v4 = 0x1.000000p1, v5 = 0.0
//     v10 = fmin v7, v6  ; v6 = 0x1.000000p0
//     v11 = fmin v8, v6  ; v6 = 0x1.000000p0
//     v12 = fmin v9, v6  ; v6 = 0x1.000000p0
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
