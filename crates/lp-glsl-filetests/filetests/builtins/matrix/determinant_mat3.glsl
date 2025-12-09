// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    mat3 m = mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
    return determinant(m);
}

// function u0:0() -> i32 system_v {
// block0:
//     v24 = iconst.i32 0x0001_0000
//     v25 = iconst.i32 0
//     v26 = iconst.i32 0
//     v27 = iconst.i32 0
//     v28 = iconst.i32 0x0001_0000
//     v29 = iconst.i32 0
//     v30 = iconst.i32 0
//     v31 = iconst.i32 0
//     v32 = iconst.i32 0x0001_0000
//     v33 = sextend.i64 v28  ; v28 = 0x0001_0000
//     v34 = sextend.i64 v32  ; v32 = 0x0001_0000
//     v35 = imul v33, v34
//     v36 = iconst.i64 16
//     v37 = sshr v35, v36  ; v36 = 16
//     v38 = ireduce.i32 v37
//     v39 = sextend.i64 v31  ; v31 = 0
//     v40 = sextend.i64 v29  ; v29 = 0
//     v41 = imul v39, v40
//     v42 = iconst.i64 16
//     v43 = sshr v41, v42  ; v42 = 16
//     v44 = ireduce.i32 v43
//     v45 = isub v38, v44
//     v46 = sextend.i64 v24  ; v24 = 0x0001_0000
//     v47 = sextend.i64 v45
//     v48 = imul v46, v47
//     v49 = iconst.i64 16
//     v50 = sshr v48, v49  ; v49 = 16
//     v51 = ireduce.i32 v50
//     v52 = sextend.i64 v25  ; v25 = 0
//     v53 = sextend.i64 v32  ; v32 = 0x0001_0000
//     v54 = imul v52, v53
//     v55 = iconst.i64 16
//     v56 = sshr v54, v55  ; v55 = 16
//     v57 = ireduce.i32 v56
//     v58 = sextend.i64 v31  ; v31 = 0
//     v59 = sextend.i64 v26  ; v26 = 0
//     v60 = imul v58, v59
//     v61 = iconst.i64 16
//     v62 = sshr v60, v61  ; v61 = 16
//     v63 = ireduce.i32 v62
//     v64 = isub v57, v63
//     v65 = sextend.i64 v27  ; v27 = 0
//     v66 = sextend.i64 v64
//     v67 = imul v65, v66
//     v68 = iconst.i64 16
//     v69 = sshr v67, v68  ; v68 = 16
//     v70 = ireduce.i32 v69
//     v71 = sextend.i64 v25  ; v25 = 0
//     v72 = sextend.i64 v29  ; v29 = 0
//     v73 = imul v71, v72
//     v74 = iconst.i64 16
//     v75 = sshr v73, v74  ; v74 = 16
//     v76 = ireduce.i32 v75
//     v77 = sextend.i64 v28  ; v28 = 0x0001_0000
//     v78 = sextend.i64 v26  ; v26 = 0
//     v79 = imul v77, v78
//     v80 = iconst.i64 16
//     v81 = sshr v79, v80  ; v80 = 16
//     v82 = ireduce.i32 v81
//     v83 = isub v76, v82
//     v84 = sextend.i64 v30  ; v30 = 0
//     v85 = sextend.i64 v83
//     v86 = imul v84, v85
//     v87 = iconst.i64 16
//     v88 = sshr v86, v87  ; v87 = 16
//     v89 = ireduce.i32 v88
//     v90 = isub v51, v70
//     v91 = iadd v90, v89
//     return v91
//
// block1:
//     v92 = iconst.i32 0
//     return v92  ; v92 = 0
// }
// run: ~= 1.0  // Identity matrix has determinant 1
