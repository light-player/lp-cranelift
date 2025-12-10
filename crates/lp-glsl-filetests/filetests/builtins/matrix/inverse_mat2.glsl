// test compile
// test run
// target riscv32.fixed32

mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return inverse(m);
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v21 = iconst.i32 0x0001_0000
//     v22 = iconst.i32 0x0002_0000
//     v23 = iconst.i32 0x0003_0000
//     v24 = iconst.i32 0x0004_0000
//     v25 = sextend.i64 v21  ; v21 = 0x0001_0000
//     v26 = sextend.i64 v24  ; v24 = 0x0004_0000
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = sextend.i64 v23  ; v23 = 0x0003_0000
//     v32 = sextend.i64 v22  ; v22 = 0x0002_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = isub v30, v36
//     v38 = iconst.i32 0x0001_0000
//     v39 = sextend.i64 v38  ; v38 = 0x0001_0000
//     v40 = iconst.i64 16
//     v41 = ishl v39, v40  ; v40 = 16
//     v42 = sextend.i64 v37
//     v43 = sdiv v41, v42
//     v44 = ireduce.i32 v43
//     v45 = iconst.i32 0
//     v46 = isub v45, v22  ; v45 = 0, v22 = 0x0002_0000
//     v47 = isub v45, v23  ; v45 = 0, v23 = 0x0003_0000
//     v48 = sextend.i64 v24  ; v24 = 0x0004_0000
//     v49 = sextend.i64 v44
//     v50 = imul v48, v49
//     v51 = iconst.i64 16
//     v52 = sshr v50, v51  ; v51 = 16
//     v53 = ireduce.i32 v52
//     v54 = sextend.i64 v46
//     v55 = sextend.i64 v44
//     v56 = imul v54, v55
//     v57 = iconst.i64 16
//     v58 = sshr v56, v57  ; v57 = 16
//     v59 = ireduce.i32 v58
//     v60 = sextend.i64 v47
//     v61 = sextend.i64 v44
//     v62 = imul v60, v61
//     v63 = iconst.i64 16
//     v64 = sshr v62, v63  ; v63 = 16
//     v65 = ireduce.i32 v64
//     v66 = sextend.i64 v21  ; v21 = 0x0001_0000
//     v67 = sextend.i64 v44
//     v68 = imul v66, v67
//     v69 = iconst.i64 16
//     v70 = sshr v68, v69  ; v69 = 16
//     v71 = ireduce.i32 v70
//     store notrap aligned v53, v0
//     store notrap aligned v59, v0+4
//     store notrap aligned v65, v0+8
//     store notrap aligned v71, v0+12
//     return
//
// block1:
//     v72 = iconst.i32 0
//     store notrap aligned v72, v0  ; v72 = 0
//     v73 = iconst.i32 0
//     store notrap aligned v73, v0+4  ; v73 = 0
//     v74 = iconst.i32 0
//     store notrap aligned v74, v0+8  ; v74 = 0
//     v75 = iconst.i32 0
//     store notrap aligned v75, v0+12  ; v75 = 0
//     return
// }
// run: ≈ mat2(-2, 1, 1.5, -0.5) (tolerance: 0.01)
