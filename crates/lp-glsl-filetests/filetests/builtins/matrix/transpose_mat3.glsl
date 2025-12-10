// test compile
// test run
// target riscv32.fixed32

mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose(m);
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v19 = iconst.i32 0x0001_0000
//     v20 = iconst.i32 0x0002_0000
//     v21 = iconst.i32 0x0003_0000
//     v22 = iconst.i32 0x0004_0000
//     v23 = iconst.i32 0x0005_0000
//     v24 = iconst.i32 0x0006_0000
//     v25 = iconst.i32 0x0007_0000
//     v26 = iconst.i32 0x0008_0000
//     v27 = iconst.i32 0x0009_0000
//     store notrap aligned v19, v0  ; v19 = 0x0001_0000
//     store notrap aligned v22, v0+4  ; v22 = 0x0004_0000
//     store notrap aligned v25, v0+8  ; v25 = 0x0007_0000
//     store notrap aligned v20, v0+12  ; v20 = 0x0002_0000
//     store notrap aligned v23, v0+16  ; v23 = 0x0005_0000
//     store notrap aligned v26, v0+20  ; v26 = 0x0008_0000
//     store notrap aligned v21, v0+24  ; v21 = 0x0003_0000
//     store notrap aligned v24, v0+28  ; v24 = 0x0006_0000
//     store notrap aligned v27, v0+32  ; v27 = 0x0009_0000
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
// run: ≈ mat3(1, 4, 7, 2, 5, 8, 3, 6, 9) (tolerance: 0.01)
