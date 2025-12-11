// test compile
// test run
// target riscv32.fixed32

mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return inverse(m);
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v6 = sextend.i64 v4  ; v4 = 0x0004_0000
//     v7 = imul v5, v6
//     v8 = iconst.i64 16
//     v9 = sshr v7, v8  ; v8 = 16
//     v10 = ireduce.i32 v9
//     v11 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v12 = sextend.i64 v2  ; v2 = 0x0002_0000
//     v13 = imul v11, v12
//     v14 = iconst.i64 16
//     v15 = sshr v13, v14  ; v14 = 16
//     v16 = ireduce.i32 v15
//     v17 = isub v10, v16
//     v18 = iconst.i32 0x0001_0000
//     v19 = iconst.i32 0
//     v20 = icmp eq v17, v19  ; v19 = 0
//     v21 = iconst.i32 0x7fff_0000
//     v22 = iconst.i32 -2147483648
//     v23 = icmp eq v18, v19  ; v18 = 0x0001_0000, v19 = 0
//     v24 = icmp slt v18, v19  ; v18 = 0x0001_0000, v19 = 0
//     v25 = select v24, v22, v21  ; v22 = -2147483648, v21 = 0x7fff_0000
//     v26 = select v23, v19, v25  ; v19 = 0
//     v27 = iconst.i32 1
//     v28 = select v20, v27, v17  ; v27 = 1
//     v29 = sextend.i64 v18  ; v18 = 0x0001_0000
//     v30 = iconst.i64 16
//     v31 = ishl v29, v30  ; v30 = 16
//     v32 = sextend.i64 v28
//     v33 = sdiv v31, v32
//     v34 = ireduce.i32 v33
//     v35 = select v20, v26, v34
//     v36 = iconst.i32 0
//     v37 = isub v36, v2  ; v36 = 0, v2 = 0x0002_0000
//     v38 = isub v36, v3  ; v36 = 0, v3 = 0x0003_0000
//     v39 = sextend.i64 v4  ; v4 = 0x0004_0000
//     v40 = sextend.i64 v35
//     v41 = imul v39, v40
//     v42 = iconst.i64 16
//     v43 = sshr v41, v42  ; v42 = 16
//     v44 = ireduce.i32 v43
//     v45 = sextend.i64 v37
//     v46 = sextend.i64 v35
//     v47 = imul v45, v46
//     v48 = iconst.i64 16
//     v49 = sshr v47, v48  ; v48 = 16
//     v50 = ireduce.i32 v49
//     v51 = sextend.i64 v38
//     v52 = sextend.i64 v35
//     v53 = imul v51, v52
//     v54 = iconst.i64 16
//     v55 = sshr v53, v54  ; v54 = 16
//     v56 = ireduce.i32 v55
//     v57 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v58 = sextend.i64 v35
//     v59 = imul v57, v58
//     v60 = iconst.i64 16
//     v61 = sshr v59, v60  ; v60 = 16
//     v62 = ireduce.i32 v61
//     store notrap aligned v44, v0
//     store notrap aligned v50, v0+4
//     store notrap aligned v56, v0+8
//     store notrap aligned v62, v0+12
//     return
//
// block1:
//     v63 = iconst.i32 0
//     store notrap aligned v63, v0  ; v63 = 0
//     v64 = iconst.i32 0
//     store notrap aligned v64, v0+4  ; v64 = 0
//     v65 = iconst.i32 0
//     store notrap aligned v65, v0+8  ; v65 = 0
//     v66 = iconst.i32 0
//     store notrap aligned v66, v0+12  ; v66 = 0
//     return
// }
// run: ≈ mat2(-2, 1, 1.5, -0.5) (tolerance: 0.01)
