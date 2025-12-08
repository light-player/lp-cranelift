// test compile
// test run

mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return inverse(m);
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = fmul v1, v4  ; v1 = 0x1.000000p0, v4 = 0x1.000000p2
//     v6 = fmul v3, v2  ; v3 = 0x1.800000p1, v2 = 0x1.000000p1
//     v7 = fsub v5, v6
//     v8 = f32const 0x1.000000p0
//     v9 = fdiv v8, v7  ; v8 = 0x1.000000p0
//     v10 = f32const 0.0
//     v11 = fsub v10, v2  ; v10 = 0.0, v2 = 0x1.000000p1
//     v12 = fsub v10, v3  ; v10 = 0.0, v3 = 0x1.800000p1
//     v13 = fmul v4, v9  ; v4 = 0x1.000000p2
//     v14 = fmul v11, v9
//     v15 = fmul v12, v9
//     v16 = fmul v1, v9  ; v1 = 0x1.000000p0
//     store notrap aligned v13, v0
//     store notrap aligned v14, v0+4
//     store notrap aligned v15, v0+8
//     store notrap aligned v16, v0+12
//     return
//
// block1:
//     v17 = f32const 0.0
//     store notrap aligned v17, v0  ; v17 = 0.0
//     v18 = f32const 0.0
//     store notrap aligned v18, v0+4  ; v18 = 0.0
//     v19 = f32const 0.0
//     store notrap aligned v19, v0+8  ; v19 = 0.0
//     v20 = f32const 0.0
//     store notrap aligned v20, v0+12  ; v20 = 0.0
//     return
// }
// run: ≈ mat2(-2, 1, 1.5, -0.5) (tolerance: 0.01)
