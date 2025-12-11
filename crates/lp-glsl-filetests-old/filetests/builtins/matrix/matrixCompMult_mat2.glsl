// test compile
// test run
// target riscv32.fixed32

mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = iconst.i32 0x0002_0000
//     v6 = iconst.i32 0x0002_0000
//     v7 = iconst.i32 0x0002_0000
//     v8 = iconst.i32 0x0002_0000
//     v9 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v10 = sextend.i64 v5  ; v5 = 0x0002_0000
//     v11 = imul v9, v10
//     v12 = iconst.i64 16
//     v13 = sshr v11, v12  ; v12 = 16
//     v14 = ireduce.i32 v13
//     v15 = sextend.i64 v2  ; v2 = 0x0002_0000
//     v16 = sextend.i64 v6  ; v6 = 0x0002_0000
//     v17 = imul v15, v16
//     v18 = iconst.i64 16
//     v19 = sshr v17, v18  ; v18 = 16
//     v20 = ireduce.i32 v19
//     v21 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v22 = sextend.i64 v7  ; v7 = 0x0002_0000
//     v23 = imul v21, v22
//     v24 = iconst.i64 16
//     v25 = sshr v23, v24  ; v24 = 16
//     v26 = ireduce.i32 v25
//     v27 = sextend.i64 v4  ; v4 = 0x0004_0000
//     v28 = sextend.i64 v8  ; v8 = 0x0002_0000
//     v29 = imul v27, v28
//     v30 = iconst.i64 16
//     v31 = sshr v29, v30  ; v30 = 16
//     v32 = ireduce.i32 v31
//     store notrap aligned v14, v0
//     store notrap aligned v20, v0+4
//     store notrap aligned v26, v0+8
//     store notrap aligned v32, v0+12
//     return
//
// block1:
//     v33 = iconst.i32 0
//     store notrap aligned v33, v0  ; v33 = 0
//     v34 = iconst.i32 0
//     store notrap aligned v34, v0+4  ; v34 = 0
//     v35 = iconst.i32 0
//     store notrap aligned v35, v0+8  ; v35 = 0
//     v36 = iconst.i32 0
//     store notrap aligned v36, v0+12  ; v36 = 0
//     return
// }
// run: ≈ mat2(2, 4, 6, 8) (tolerance: 0.01)
