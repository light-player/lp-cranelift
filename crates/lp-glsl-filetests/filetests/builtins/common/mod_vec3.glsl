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
//     v0 = iconst.i32 0x0007_0000
//     v1 = iconst.i32 0x0008_0000
//     v2 = iconst.i32 0x0009_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0003_0000
//     v5 = iconst.i32 0x0004_0000
//     v6 = iconst.i32 0
//     v7 = icmp eq v3, v6  ; v3 = 0x0003_0000, v6 = 0
//     v8 = iconst.i32 0x7fff_0000
//     v9 = iconst.i32 -2147483648
//     v10 = icmp eq v0, v6  ; v0 = 0x0007_0000, v6 = 0
//     v11 = icmp slt v0, v6  ; v0 = 0x0007_0000, v6 = 0
//     v12 = select v11, v9, v8  ; v9 = -2147483648, v8 = 0x7fff_0000
//     v13 = select v10, v6, v12  ; v6 = 0
//     v14 = iconst.i32 1
//     v15 = select v7, v14, v3  ; v14 = 1, v3 = 0x0003_0000
//     v16 = sextend.i64 v0  ; v0 = 0x0007_0000
//     v17 = iconst.i64 16
//     v18 = ishl v16, v17  ; v17 = 16
//     v19 = sextend.i64 v15
//     v20 = sdiv v18, v19
//     v21 = ireduce.i32 v20
//     v22 = select v7, v13, v21
//     v23 = iconst.i32 16
//     v24 = sshr v22, v23  ; v23 = 16
//     v25 = ishl v24, v23  ; v23 = 16
//     v26 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v27 = sextend.i64 v25
//     v28 = imul v26, v27
//     v29 = iconst.i64 16
//     v30 = sshr v28, v29  ; v29 = 16
//     v31 = ireduce.i32 v30
//     v32 = isub v0, v31  ; v0 = 0x0007_0000
//     v33 = iconst.i32 0
//     v34 = icmp eq v4, v33  ; v4 = 0x0003_0000, v33 = 0
//     v35 = iconst.i32 0x7fff_0000
//     v36 = iconst.i32 -2147483648
//     v37 = icmp eq v1, v33  ; v1 = 0x0008_0000, v33 = 0
//     v38 = icmp slt v1, v33  ; v1 = 0x0008_0000, v33 = 0
//     v39 = select v38, v36, v35  ; v36 = -2147483648, v35 = 0x7fff_0000
//     v40 = select v37, v33, v39  ; v33 = 0
//     v41 = iconst.i32 1
//     v42 = select v34, v41, v4  ; v41 = 1, v4 = 0x0003_0000
//     v43 = sextend.i64 v1  ; v1 = 0x0008_0000
//     v44 = iconst.i64 16
//     v45 = ishl v43, v44  ; v44 = 16
//     v46 = sextend.i64 v42
//     v47 = sdiv v45, v46
//     v48 = ireduce.i32 v47
//     v49 = select v34, v40, v48
//     v50 = iconst.i32 16
//     v51 = sshr v49, v50  ; v50 = 16
//     v52 = ishl v51, v50  ; v50 = 16
//     v53 = sextend.i64 v4  ; v4 = 0x0003_0000
//     v54 = sextend.i64 v52
//     v55 = imul v53, v54
//     v56 = iconst.i64 16
//     v57 = sshr v55, v56  ; v56 = 16
//     v58 = ireduce.i32 v57
//     v59 = isub v1, v58  ; v1 = 0x0008_0000
//     v60 = iconst.i32 0
//     v61 = icmp eq v5, v60  ; v5 = 0x0004_0000, v60 = 0
//     v62 = iconst.i32 0x7fff_0000
//     v63 = iconst.i32 -2147483648
//     v64 = icmp eq v2, v60  ; v2 = 0x0009_0000, v60 = 0
//     v65 = icmp slt v2, v60  ; v2 = 0x0009_0000, v60 = 0
//     v66 = select v65, v63, v62  ; v63 = -2147483648, v62 = 0x7fff_0000
//     v67 = select v64, v60, v66  ; v60 = 0
//     v68 = iconst.i32 1
//     v69 = select v61, v68, v5  ; v68 = 1, v5 = 0x0004_0000
//     v70 = sextend.i64 v2  ; v2 = 0x0009_0000
//     v71 = iconst.i64 16
//     v72 = ishl v70, v71  ; v71 = 16
//     v73 = sextend.i64 v69
//     v74 = sdiv v72, v73
//     v75 = ireduce.i32 v74
//     v76 = select v61, v67, v75
//     v77 = iconst.i32 16
//     v78 = sshr v76, v77  ; v77 = 16
//     v79 = ishl v78, v77  ; v77 = 16
//     v80 = sextend.i64 v5  ; v5 = 0x0004_0000
//     v81 = sextend.i64 v79
//     v82 = imul v80, v81
//     v83 = iconst.i64 16
//     v84 = sshr v82, v83  ; v83 = 16
//     v85 = ireduce.i32 v84
//     v86 = isub v2, v85  ; v2 = 0x0009_0000
//     v87 = iadd v32, v59
//     v88 = iadd v87, v86
//     v89 = iconst.i32 0x0003_fd71
//     v90 = icmp sgt v88, v89  ; v89 = 0x0003_fd71
//     v91 = sextend.i32 v90
//     v92 = iconst.i8 1
//     v93 = iconst.i8 0
//     v94 = select v91, v92, v93  ; v92 = 1, v93 = 0
//     v95 = iconst.i32 0x0004_028f
//     v96 = icmp slt v88, v95  ; v95 = 0x0004_028f
//     v97 = sextend.i32 v96
//     v98 = iconst.i8 1
//     v99 = iconst.i8 0
//     v100 = select v97, v98, v99  ; v98 = 1, v99 = 0
//     v101 = iconst.i8 0
//     v102 = iconst.i8 1
//     v103 = icmp ne v94, v101  ; v101 = 0
//     v104 = icmp ne v100, v101  ; v101 = 0
//     v105 = select v104, v102, v101  ; v102 = 1, v101 = 0
//     v106 = select v103, v105, v101  ; v101 = 0
//     return v106
//
// block1:
//     v107 = iconst.i8 0
//     return v107  ; v107 = 0
// }
// run: == true
