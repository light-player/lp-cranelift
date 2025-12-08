// test compile
// test run

mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose(m);
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = f32const 0x1.400000p2
//     v6 = f32const 0x1.800000p2
//     v7 = f32const 0x1.c00000p2
//     v8 = f32const 0x1.000000p3
//     v9 = f32const 0x1.200000p3
//     store notrap aligned v1, v0  ; v1 = 0x1.000000p0
//     store notrap aligned v4, v0+4  ; v4 = 0x1.000000p2
//     store notrap aligned v7, v0+8  ; v7 = 0x1.c00000p2
//     store notrap aligned v2, v0+12  ; v2 = 0x1.000000p1
//     store notrap aligned v5, v0+16  ; v5 = 0x1.400000p2
//     store notrap aligned v8, v0+20  ; v8 = 0x1.000000p3
//     store notrap aligned v3, v0+24  ; v3 = 0x1.800000p1
//     store notrap aligned v6, v0+28  ; v6 = 0x1.800000p2
//     store notrap aligned v9, v0+32  ; v9 = 0x1.200000p3
//     return
//
// block1:
//     v10 = f32const 0.0
//     store notrap aligned v10, v0  ; v10 = 0.0
//     v11 = f32const 0.0
//     store notrap aligned v11, v0+4  ; v11 = 0.0
//     v12 = f32const 0.0
//     store notrap aligned v12, v0+8  ; v12 = 0.0
//     v13 = f32const 0.0
//     store notrap aligned v13, v0+12  ; v13 = 0.0
//     v14 = f32const 0.0
//     store notrap aligned v14, v0+16  ; v14 = 0.0
//     v15 = f32const 0.0
//     store notrap aligned v15, v0+20  ; v15 = 0.0
//     v16 = f32const 0.0
//     store notrap aligned v16, v0+24  ; v16 = 0.0
//     v17 = f32const 0.0
//     store notrap aligned v17, v0+28  ; v17 = 0.0
//     v18 = f32const 0.0
//     store notrap aligned v18, v0+32  ; v18 = 0.0
//     return
// }
// run: ≈ mat3(1, 4, 7, 2, 5, 8, 3, 6, 9) (tolerance: 0.01)
