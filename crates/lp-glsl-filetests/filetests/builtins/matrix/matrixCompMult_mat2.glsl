// test compile
// test run
// target riscv32.fixed32

mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v17 = iconst.i32 0x0001_0000
//     v18 = iconst.i32 0x0002_0000
//     v19 = iconst.i32 0x0003_0000
//     v20 = iconst.i32 0x0004_0000
//     v21 = iconst.i32 0x0002_0000
//     v22 = iconst.i32 0x0002_0000
//     v23 = iconst.i32 0x0002_0000
//     v24 = iconst.i32 0x0002_0000
//     v25 = sextend.i64 v17  ; v17 = 0x0001_0000
//     v26 = sextend.i64 v21  ; v21 = 0x0002_0000
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = sextend.i64 v18  ; v18 = 0x0002_0000
//     v32 = sextend.i64 v22  ; v22 = 0x0002_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = sextend.i64 v19  ; v19 = 0x0003_0000
//     v38 = sextend.i64 v23  ; v23 = 0x0002_0000
//     v39 = imul v37, v38
//     v40 = iconst.i64 16
//     v41 = sshr v39, v40  ; v40 = 16
//     v42 = ireduce.i32 v41
//     v43 = sextend.i64 v20  ; v20 = 0x0004_0000
//     v44 = sextend.i64 v24  ; v24 = 0x0002_0000
//     v45 = imul v43, v44
//     v46 = iconst.i64 16
//     v47 = sshr v45, v46  ; v46 = 16
//     v48 = ireduce.i32 v47
//     store notrap aligned v30, v0
//     store notrap aligned v36, v0+4
//     store notrap aligned v42, v0+8
//     store notrap aligned v48, v0+12
//     return
//
// block1:
//     v49 = iconst.i32 0
//     store notrap aligned v49, v0  ; v49 = 0
//     v50 = iconst.i32 0
//     store notrap aligned v50, v0+4  ; v50 = 0
//     v51 = iconst.i32 0
//     store notrap aligned v51, v0+8  ; v51 = 0
//     v52 = iconst.i32 0
//     store notrap aligned v52, v0+12  ; v52 = 0
//     return
// }
// run: ≈ mat2(2, 4, 6, 8) (tolerance: 0.01)
