// test compile
// test run

mat3 main() {
    vec3 u = vec3(1.0, 2.0, 3.0);
    vec3 v = vec3(4.0, 5.0, 6.0);
    return outerProduct(u, v);
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = f32const 0x1.400000p2
//     v6 = f32const 0x1.800000p2
//     v7 = fmul v1, v4  ; v1 = 0x1.000000p0, v4 = 0x1.000000p2
//     v8 = fmul v2, v4  ; v2 = 0x1.000000p1, v4 = 0x1.000000p2
//     v9 = fmul v3, v4  ; v3 = 0x1.800000p1, v4 = 0x1.000000p2
//     v10 = fmul v1, v5  ; v1 = 0x1.000000p0, v5 = 0x1.400000p2
//     v11 = fmul v2, v5  ; v2 = 0x1.000000p1, v5 = 0x1.400000p2
//     v12 = fmul v3, v5  ; v3 = 0x1.800000p1, v5 = 0x1.400000p2
//     v13 = fmul v1, v6  ; v1 = 0x1.000000p0, v6 = 0x1.800000p2
//     v14 = fmul v2, v6  ; v2 = 0x1.000000p1, v6 = 0x1.800000p2
//     v15 = fmul v3, v6  ; v3 = 0x1.800000p1, v6 = 0x1.800000p2
//     store notrap aligned v7, v0
//     store notrap aligned v8, v0+4
//     store notrap aligned v9, v0+8
//     store notrap aligned v10, v0+12
//     store notrap aligned v11, v0+16
//     store notrap aligned v12, v0+20
//     store notrap aligned v13, v0+24
//     store notrap aligned v14, v0+28
//     store notrap aligned v15, v0+32
//     return
//
// block1:
//     v16 = f32const 0.0
//     store notrap aligned v16, v0  ; v16 = 0.0
//     v17 = f32const 0.0
//     store notrap aligned v17, v0+4  ; v17 = 0.0
//     v18 = f32const 0.0
//     store notrap aligned v18, v0+8  ; v18 = 0.0
//     v19 = f32const 0.0
//     store notrap aligned v19, v0+12  ; v19 = 0.0
//     v20 = f32const 0.0
//     store notrap aligned v20, v0+16  ; v20 = 0.0
//     v21 = f32const 0.0
//     store notrap aligned v21, v0+20  ; v21 = 0.0
//     v22 = f32const 0.0
//     store notrap aligned v22, v0+24  ; v22 = 0.0
//     v23 = f32const 0.0
//     store notrap aligned v23, v0+28  ; v23 = 0.0
//     v24 = f32const 0.0
//     store notrap aligned v24, v0+32  ; v24 = 0.0
//     return
// }
// run: ≈ mat3(4, 8, 12, 5, 10, 15, 6, 12, 18) (tolerance: 0.01)
