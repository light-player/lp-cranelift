// test compile
// test run
// target riscv32.fixed32

vec3 main() {
    return sqrt(vec3(4.0, 9.0, 16.0));  // Should return (2.0, 3.0, 4.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v10 = iconst.i32 0x0004_0000
//     v11 = iconst.i32 0x0009_0000
//     v12 = iconst.i32 0x0010_0000
//     v13 = iconst.i32 0
//     v14 = icmp eq v10, v13  ; v10 = 0x0004_0000, v13 = 0
//     v15 = iconst.i32 8
//     v16 = sshr v10, v15  ; v10 = 0x0004_0000, v15 = 8
//     v17 = sextend.i64 v10  ; v10 = 0x0004_0000
//     v18 = sextend.i64 v16
//     v19 = ishl v17, v15  ; v15 = 8
//     v20 = sdiv v19, v18
//     v21 = iadd v18, v20
//     v22 = iconst.i64 1
//     v23 = sshr v21, v22  ; v22 = 1
//     v24 = sdiv v19, v23
//     v25 = iadd v23, v24
//     v26 = iconst.i64 1
//     v27 = sshr v25, v26  ; v26 = 1
//     v28 = sdiv v19, v27
//     v29 = iadd v27, v28
//     v30 = iconst.i64 1
//     v31 = sshr v29, v30  ; v30 = 1
//     v32 = sdiv v19, v31
//     v33 = iadd v31, v32
//     v34 = iconst.i64 1
//     v35 = sshr v33, v34  ; v34 = 1
//     v36 = sdiv v19, v35
//     v37 = iadd v35, v36
//     v38 = iconst.i64 1
//     v39 = sshr v37, v38  ; v38 = 1
//     v40 = ireduce.i32 v39
//     v41 = select v14, v13, v40  ; v13 = 0
//     v42 = iconst.i32 0
//     v43 = icmp eq v11, v42  ; v11 = 0x0009_0000, v42 = 0
//     v44 = iconst.i32 8
//     v45 = sshr v11, v44  ; v11 = 0x0009_0000, v44 = 8
//     v46 = sextend.i64 v11  ; v11 = 0x0009_0000
//     v47 = sextend.i64 v45
//     v48 = ishl v46, v44  ; v44 = 8
//     v49 = sdiv v48, v47
//     v50 = iadd v47, v49
//     v51 = iconst.i64 1
//     v52 = sshr v50, v51  ; v51 = 1
//     v53 = sdiv v48, v52
//     v54 = iadd v52, v53
//     v55 = iconst.i64 1
//     v56 = sshr v54, v55  ; v55 = 1
//     v57 = sdiv v48, v56
//     v58 = iadd v56, v57
//     v59 = iconst.i64 1
//     v60 = sshr v58, v59  ; v59 = 1
//     v61 = sdiv v48, v60
//     v62 = iadd v60, v61
//     v63 = iconst.i64 1
//     v64 = sshr v62, v63  ; v63 = 1
//     v65 = sdiv v48, v64
//     v66 = iadd v64, v65
//     v67 = iconst.i64 1
//     v68 = sshr v66, v67  ; v67 = 1
//     v69 = ireduce.i32 v68
//     v70 = select v43, v42, v69  ; v42 = 0
//     v71 = iconst.i32 0
//     v72 = icmp eq v12, v71  ; v12 = 0x0010_0000, v71 = 0
//     v73 = iconst.i32 8
//     v74 = sshr v12, v73  ; v12 = 0x0010_0000, v73 = 8
//     v75 = sextend.i64 v12  ; v12 = 0x0010_0000
//     v76 = sextend.i64 v74
//     v77 = ishl v75, v73  ; v73 = 8
//     v78 = sdiv v77, v76
//     v79 = iadd v76, v78
//     v80 = iconst.i64 1
//     v81 = sshr v79, v80  ; v80 = 1
//     v82 = sdiv v77, v81
//     v83 = iadd v81, v82
//     v84 = iconst.i64 1
//     v85 = sshr v83, v84  ; v84 = 1
//     v86 = sdiv v77, v85
//     v87 = iadd v85, v86
//     v88 = iconst.i64 1
//     v89 = sshr v87, v88  ; v88 = 1
//     v90 = sdiv v77, v89
//     v91 = iadd v89, v90
//     v92 = iconst.i64 1
//     v93 = sshr v91, v92  ; v92 = 1
//     v94 = sdiv v77, v93
//     v95 = iadd v93, v94
//     v96 = iconst.i64 1
//     v97 = sshr v95, v96  ; v96 = 1
//     v98 = ireduce.i32 v97
//     v99 = select v72, v71, v98  ; v71 = 0
//     store notrap aligned v41, v0
//     store notrap aligned v70, v0+4
//     store notrap aligned v99, v0+8
//     return
//
// block1:
//     v100 = iconst.i32 0
//     store notrap aligned v100, v0  ; v100 = 0
//     v101 = iconst.i32 0
//     store notrap aligned v101, v0+4  ; v101 = 0
//     v102 = iconst.i32 0
//     store notrap aligned v102, v0+8  ; v102 = 0
//     return
// }
// run: ≈ vec3(2, 3, 4) (tolerance: 0.01)
