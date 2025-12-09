// test compile

// target riscv32
vec3 main() {
    vec3 x = vec3(1.0, 0.0, 0.0);
    vec3 y = vec3(0.0, 1.0, 0.0);
    return cross(x, y);  // = (0.0, 0.0, 1.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0.0
//     v3 = f32const 0.0
//     v4 = f32const 0.0
//     v5 = f32const 0x1.000000p0
//     v6 = f32const 0.0
//     v7 = fmul v2, v6  ; v2 = 0.0, v6 = 0.0
//     v8 = fmul v3, v5  ; v3 = 0.0, v5 = 0x1.000000p0
//     v9 = fsub v7, v8
//     v10 = fmul v3, v4  ; v3 = 0.0, v4 = 0.0
//     v11 = fmul v1, v6  ; v1 = 0x1.000000p0, v6 = 0.0
//     v12 = fsub v10, v11
//     v13 = fmul v1, v5  ; v1 = 0x1.000000p0, v5 = 0x1.000000p0
//     v14 = fmul v2, v4  ; v2 = 0.0, v4 = 0.0
//     v15 = fsub v13, v14
//     store notrap aligned v9, v0
//     store notrap aligned v12, v0+4
//     store notrap aligned v15, v0+8
//     return
//
// block1:
//     v16 = f32const 0.0
//     store notrap aligned v16, v0  ; v16 = 0.0
//     v17 = f32const 0.0
//     store notrap aligned v17, v0+4  ; v17 = 0.0
//     v18 = f32const 0.0
//     store notrap aligned v18, v0+8  ; v18 = 0.0
//     return
// }
