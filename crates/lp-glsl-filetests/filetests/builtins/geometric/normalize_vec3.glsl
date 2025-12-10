// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 v = vec3(3.0, 0.0, 4.0);  // length = 5.0
    return normalize(v);  // (0.6, 0.0, 0.8)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v16 = iconst.i32 0x0003_0000
//     v17 = iconst.i32 0
//     v18 = iconst.i32 0x0004_0000
//     v19 = sextend.i64 v16  ; v16 = 0x0003_0000
//     v20 = sextend.i64 v16  ; v16 = 0x0003_0000
//     v21 = imul v19, v20
//     v22 = iconst.i64 16
//     v23 = sshr v21, v22  ; v22 = 16
//     v24 = ireduce.i32 v23
//     v25 = sextend.i64 v17  ; v17 = 0
//     v26 = sextend.i64 v17  ; v17 = 0
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = iadd v24, v30
//     v32 = sextend.i64 v18  ; v18 = 0x0004_0000
//     v33 = sextend.i64 v18  ; v18 = 0x0004_0000
//     v34 = imul v32, v33
//     v35 = iconst.i64 16
//     v36 = sshr v34, v35  ; v35 = 16
//     v37 = ireduce.i32 v36
//     v38 = iadd v31, v37
//     v39 = iconst.i32 0
//     v40 = icmp eq v38, v39  ; v39 = 0
//     v41 = iconst.i32 8
//     v42 = sshr v38, v41  ; v41 = 8
//     v43 = sextend.i64 v38
//     v44 = sextend.i64 v42
//     v45 = ishl v43, v41  ; v41 = 8
//     v46 = sdiv v45, v44
//     v47 = iadd v44, v46
//     v48 = iconst.i64 1
//     v49 = sshr v47, v48  ; v48 = 1
//     v50 = sdiv v45, v49
//     v51 = iadd v49, v50
//     v52 = iconst.i64 1
//     v53 = sshr v51, v52  ; v52 = 1
//     v54 = sdiv v45, v53
//     v55 = iadd v53, v54
//     v56 = iconst.i64 1
//     v57 = sshr v55, v56  ; v56 = 1
//     v58 = sdiv v45, v57
//     v59 = iadd v57, v58
//     v60 = iconst.i64 1
//     v61 = sshr v59, v60  ; v60 = 1
//     v62 = sdiv v45, v61
//     v63 = iadd v61, v62
//     v64 = iconst.i64 1
//     v65 = sshr v63, v64  ; v64 = 1
//     v66 = ireduce.i32 v65
//     v67 = select v40, v39, v66  ; v39 = 0
//     v68 = sextend.i64 v16  ; v16 = 0x0003_0000
//     v69 = iconst.i64 16
//     v70 = ishl v68, v69  ; v69 = 16
//     v71 = sextend.i64 v67
//     v72 = sdiv v70, v71
//     v73 = ireduce.i32 v72
//     v74 = sextend.i64 v17  ; v17 = 0
//     v75 = iconst.i64 16
//     v76 = ishl v74, v75  ; v75 = 16
//     v77 = sextend.i64 v67
//     v78 = sdiv v76, v77
//     v79 = ireduce.i32 v78
//     v80 = sextend.i64 v18  ; v18 = 0x0004_0000
//     v81 = iconst.i64 16
//     v82 = ishl v80, v81  ; v81 = 16
//     v83 = sextend.i64 v67
//     v84 = sdiv v82, v83
//     v85 = ireduce.i32 v84
//     store notrap aligned v73, v0
//     store notrap aligned v79, v0+4
//     store notrap aligned v85, v0+8
//     return
//
// block1:
//     v86 = iconst.i32 0
//     store notrap aligned v86, v0  ; v86 = 0
//     v87 = iconst.i32 0
//     store notrap aligned v87, v0+4  ; v87 = 0
//     v88 = iconst.i32 0
//     store notrap aligned v88, v0+8  ; v88 = 0
//     return
// }
