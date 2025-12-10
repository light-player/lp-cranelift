// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 result = mod(vec3(7.0, 8.0, 9.0), vec3(3.0, 3.0, 4.0));  // (1.0, 2.0, 1.0)
    // Validate: sum = 1 + 2 + 1 = 4.0
    float sum = result.x + result.y + result.z;
    return sum > 3.99 && sum < 4.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v37 = iconst.i32 0x0007_0000
//     v38 = iconst.i32 0x0008_0000
//     v39 = iconst.i32 0x0009_0000
//     v40 = iconst.i32 0x0003_0000
//     v41 = iconst.i32 0x0003_0000
//     v42 = iconst.i32 0x0004_0000
//     v43 = sextend.i64 v37  ; v37 = 0x0007_0000
//     v44 = iconst.i64 16
//     v45 = ishl v43, v44  ; v44 = 16
//     v46 = sextend.i64 v40  ; v40 = 0x0003_0000
//     v47 = sdiv v45, v46
//     v48 = ireduce.i32 v47
//     v49 = iconst.i64 16
//     v50 = sextend.i64 v48
//     v51 = sshr v50, v49  ; v49 = 16
//     v52 = ishl v51, v49  ; v49 = 16
//     v53 = ireduce.i32 v52
//     v54 = sextend.i64 v40  ; v40 = 0x0003_0000
//     v55 = sextend.i64 v53
//     v56 = imul v54, v55
//     v57 = iconst.i64 16
//     v58 = sshr v56, v57  ; v57 = 16
//     v59 = ireduce.i32 v58
//     v60 = isub v37, v59  ; v37 = 0x0007_0000
//     v61 = sextend.i64 v38  ; v38 = 0x0008_0000
//     v62 = iconst.i64 16
//     v63 = ishl v61, v62  ; v62 = 16
//     v64 = sextend.i64 v41  ; v41 = 0x0003_0000
//     v65 = sdiv v63, v64
//     v66 = ireduce.i32 v65
//     v67 = iconst.i64 16
//     v68 = sextend.i64 v66
//     v69 = sshr v68, v67  ; v67 = 16
//     v70 = ishl v69, v67  ; v67 = 16
//     v71 = ireduce.i32 v70
//     v72 = sextend.i64 v41  ; v41 = 0x0003_0000
//     v73 = sextend.i64 v71
//     v74 = imul v72, v73
//     v75 = iconst.i64 16
//     v76 = sshr v74, v75  ; v75 = 16
//     v77 = ireduce.i32 v76
//     v78 = isub v38, v77  ; v38 = 0x0008_0000
//     v79 = sextend.i64 v39  ; v39 = 0x0009_0000
//     v80 = iconst.i64 16
//     v81 = ishl v79, v80  ; v80 = 16
//     v82 = sextend.i64 v42  ; v42 = 0x0004_0000
//     v83 = sdiv v81, v82
//     v84 = ireduce.i32 v83
//     v85 = iconst.i64 16
//     v86 = sextend.i64 v84
//     v87 = sshr v86, v85  ; v85 = 16
//     v88 = ishl v87, v85  ; v85 = 16
//     v89 = ireduce.i32 v88
//     v90 = sextend.i64 v42  ; v42 = 0x0004_0000
//     v91 = sextend.i64 v89
//     v92 = imul v90, v91
//     v93 = iconst.i64 16
//     v94 = sshr v92, v93  ; v93 = 16
//     v95 = ireduce.i32 v94
//     v96 = isub v39, v95  ; v39 = 0x0009_0000
//     v97 = iadd v60, v78
//     v98 = iadd v97, v96
//     v99 = iconst.i32 0x0003_fd71
//     v100 = icmp sgt v98, v99  ; v99 = 0x0003_fd71
//     v22 = iconst.i8 1
//     v23 = iconst.i8 0
//     v24 = select v100, v22, v23  ; v22 = 1, v23 = 0
//     v101 = iconst.i32 0x0004_028f
//     v102 = icmp slt v98, v101  ; v101 = 0x0004_028f
//     v27 = iconst.i8 1
//     v28 = iconst.i8 0
//     v29 = select v102, v27, v28  ; v27 = 1, v28 = 0
//     v30 = iconst.i8 0
//     v31 = iconst.i8 1
//     v32 = icmp ne v24, v30  ; v30 = 0
//     v33 = icmp ne v29, v30  ; v30 = 0
//     v34 = select v33, v31, v30  ; v31 = 1, v30 = 0
//     v35 = select v32, v34, v30  ; v30 = 0
//     return v35
//
// block1:
//     v36 = iconst.i8 0
//     return v36  ; v36 = 0
// }
// run: == true
