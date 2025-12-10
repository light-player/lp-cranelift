// test riscv32.fixed32
// test run

mat3 main() {
    return mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0) + mat3(0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9);
}

// Generated CLIF
// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = f32const 0x1.400000p2
//     v6 = f32const 0x1.800000p2
//     v7 = f32const 0x1.c00000p2
//     v8 = f32const 0x1.000000p3
//     v9 = f32const 0x1.200000p3
//     v10 = f32const 0x1.99999ap-4
//     v11 = f32const 0x1.99999ap-3
//     v12 = f32const 0x1.333334p-2
//     v13 = f32const 0x1.99999ap-2
//     v14 = f32const 0x1.000000p-1
//     v15 = f32const 0x1.333334p-1
//     v16 = f32const 0x1.666666p-1
//     v17 = f32const 0x1.99999ap-1
//     v18 = f32const 0x1.ccccccp-1
//     v19 = fadd v1, v10  ; v1 = 0x1.000000p0, v10 = 0x1.99999ap-4
//     v20 = fadd v2, v11  ; v2 = 0x1.000000p1, v11 = 0x1.99999ap-3
//     v21 = fadd v3, v12  ; v3 = 0x1.800000p1, v12 = 0x1.333334p-2
//     v22 = fadd v4, v13  ; v4 = 0x1.000000p2, v13 = 0x1.99999ap-2
//     v23 = fadd v5, v14  ; v5 = 0x1.400000p2, v14 = 0x1.000000p-1
//     v24 = fadd v6, v15  ; v6 = 0x1.800000p2, v15 = 0x1.333334p-1
//     v25 = fadd v7, v16  ; v7 = 0x1.c00000p2, v16 = 0x1.666666p-1
//     v26 = fadd v8, v17  ; v8 = 0x1.000000p3, v17 = 0x1.99999ap-1
//     v27 = fadd v9, v18  ; v9 = 0x1.200000p3, v18 = 0x1.ccccccp-1
//     store notrap aligned v19, v0
//     store notrap aligned v20, v0+4
//     store notrap aligned v21, v0+8
//     store notrap aligned v22, v0+12
//     store notrap aligned v23, v0+16
//     store notrap aligned v24, v0+20
//     store notrap aligned v25, v0+24
//     store notrap aligned v26, v0+28
//     store notrap aligned v27, v0+32
//     return
//
// block1:
//     v28 = f32const 0.0
//     store notrap aligned v28, v0  ; v28 = 0.0
//     v29 = f32const 0.0
//     store notrap aligned v29, v0+4  ; v29 = 0.0
//     v30 = f32const 0.0
//     store notrap aligned v30, v0+8  ; v30 = 0.0
//     v31 = f32const 0.0
//     store notrap aligned v31, v0+12  ; v31 = 0.0
//     v32 = f32const 0.0
//     store notrap aligned v32, v0+16  ; v32 = 0.0
//     v33 = f32const 0.0
//     store notrap aligned v33, v0+20  ; v33 = 0.0
//     v34 = f32const 0.0
//     store notrap aligned v34, v0+24  ; v34 = 0.0
//     v35 = f32const 0.0
//     store notrap aligned v35, v0+28  ; v35 = 0.0
//     v36 = f32const 0.0
//     store notrap aligned v36, v0+32  ; v36 = 0.0
//     return
// }
//
// Transformed CLIF
// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = iconst.i32 0x0005_0000
//     v6 = iconst.i32 0x0006_0000
//     v7 = iconst.i32 0x0007_0000
//     v8 = iconst.i32 0x0008_0000
//     v9 = iconst.i32 0x0009_0000
//     v10 = iconst.i32 6554
//     v11 = iconst.i32 0x3333
//     v12 = iconst.i32 0x4ccd
//     v13 = iconst.i32 0x6666
//     v14 = iconst.i32 0x8000
//     v15 = iconst.i32 0x999a
//     v16 = iconst.i32 0xb333
//     v17 = iconst.i32 0xcccd
//     v18 = iconst.i32 0xe666
//     v19 = iadd v1, v10  ; v1 = 0x0001_0000, v10 = 6554
//     v20 = iadd v2, v11  ; v2 = 0x0002_0000, v11 = 0x3333
//     v21 = iadd v3, v12  ; v3 = 0x0003_0000, v12 = 0x4ccd
//     v22 = iadd v4, v13  ; v4 = 0x0004_0000, v13 = 0x6666
//     v23 = iadd v5, v14  ; v5 = 0x0005_0000, v14 = 0x8000
//     v24 = iadd v6, v15  ; v6 = 0x0006_0000, v15 = 0x999a
//     v25 = iadd v7, v16  ; v7 = 0x0007_0000, v16 = 0xb333
//     v26 = iadd v8, v17  ; v8 = 0x0008_0000, v17 = 0xcccd
//     v27 = iadd v9, v18  ; v9 = 0x0009_0000, v18 = 0xe666
//     store notrap aligned v19, v0
//     store notrap aligned v20, v0+4
//     store notrap aligned v21, v0+8
//     store notrap aligned v22, v0+12
//     store notrap aligned v23, v0+16
//     store notrap aligned v24, v0+20
//     store notrap aligned v25, v0+24
//     store notrap aligned v26, v0+28
//     store notrap aligned v27, v0+32
//     return
//
// block1:
//     v28 = iconst.i32 0
//     store notrap aligned v28, v0  ; v28 = 0
//     v29 = iconst.i32 0
//     store notrap aligned v29, v0+4  ; v29 = 0
//     v30 = iconst.i32 0
//     store notrap aligned v30, v0+8  ; v30 = 0
//     v31 = iconst.i32 0
//     store notrap aligned v31, v0+12  ; v31 = 0
//     v32 = iconst.i32 0
//     store notrap aligned v32, v0+16  ; v32 = 0
//     v33 = iconst.i32 0
//     store notrap aligned v33, v0+20  ; v33 = 0
//     v34 = iconst.i32 0
//     store notrap aligned v34, v0+24  ; v34 = 0
//     v35 = iconst.i32 0
//     store notrap aligned v35, v0+28  ; v35 = 0
//     v36 = iconst.i32 0
//     store notrap aligned v36, v0+32  ; v36 = 0
//     return
// }
// run: ≈ mat3(1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8, 9.9)
