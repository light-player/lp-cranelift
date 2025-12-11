// test compile
// test run
// target riscv32.fixed32

mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose(m);
}

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
//     store notrap aligned v1, v0  ; v1 = 0x0001_0000
//     store notrap aligned v4, v0+4  ; v4 = 0x0004_0000
//     store notrap aligned v7, v0+8  ; v7 = 0x0007_0000
//     store notrap aligned v2, v0+12  ; v2 = 0x0002_0000
//     store notrap aligned v5, v0+16  ; v5 = 0x0005_0000
//     store notrap aligned v8, v0+20  ; v8 = 0x0008_0000
//     store notrap aligned v3, v0+24  ; v3 = 0x0003_0000
//     store notrap aligned v6, v0+28  ; v6 = 0x0006_0000
//     store notrap aligned v9, v0+32  ; v9 = 0x0009_0000
//     return
//
// block1:
//     v10 = iconst.i32 0
//     store notrap aligned v10, v0  ; v10 = 0
//     v11 = iconst.i32 0
//     store notrap aligned v11, v0+4  ; v11 = 0
//     v12 = iconst.i32 0
//     store notrap aligned v12, v0+8  ; v12 = 0
//     v13 = iconst.i32 0
//     store notrap aligned v13, v0+12  ; v13 = 0
//     v14 = iconst.i32 0
//     store notrap aligned v14, v0+16  ; v14 = 0
//     v15 = iconst.i32 0
//     store notrap aligned v15, v0+20  ; v15 = 0
//     v16 = iconst.i32 0
//     store notrap aligned v16, v0+24  ; v16 = 0
//     v17 = iconst.i32 0
//     store notrap aligned v17, v0+28  ; v17 = 0
//     v18 = iconst.i32 0
//     store notrap aligned v18, v0+32  ; v18 = 0
//     return
// }
// run: ≈ mat3(1, 4, 7, 2, 5, 8, 3, 6, 9) (tolerance: 0.01)
