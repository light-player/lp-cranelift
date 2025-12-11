// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 result = mod(vec3(7.0, 8.0, 9.0), 3.0);  // (1.0, 2.0, 0.0)
    // Validate: sum = 1 + 2 + 0 = 3.0
    float sum = result.x + result.y + result.z;
    return sum > 2.99 && sum < 3.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0x0007_0000
//     v1 = iconst.i32 0x0008_0000
//     v2 = iconst.i32 0x0009_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0
//     v5 = icmp eq v3, v4  ; v3 = 0x0003_0000, v4 = 0
//     v6 = iconst.i32 0x7fff_0000
//     v7 = iconst.i32 -2147483648
//     v8 = icmp eq v0, v4  ; v0 = 0x0007_0000, v4 = 0
//     v9 = icmp slt v0, v4  ; v0 = 0x0007_0000, v4 = 0
//     v10 = select v9, v7, v6  ; v7 = -2147483648, v6 = 0x7fff_0000
//     v11 = select v8, v4, v10  ; v4 = 0
//     v12 = iconst.i32 1
//     v13 = select v5, v12, v3  ; v12 = 1, v3 = 0x0003_0000
//     v14 = sextend.i64 v0  ; v0 = 0x0007_0000
//     v15 = iconst.i64 16
//     v16 = ishl v14, v15  ; v15 = 16
//     v17 = sextend.i64 v13
//     v18 = sdiv v16, v17
//     v19 = ireduce.i32 v18
//     v20 = select v5, v11, v19
//     v21 = iconst.i32 16
//     v22 = sshr v20, v21  ; v21 = 16
//     v23 = ishl v22, v21  ; v21 = 16
//     v24 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v25 = sextend.i64 v23
//     v26 = imul v24, v25
//     v27 = iconst.i64 16
//     v28 = sshr v26, v27  ; v27 = 16
//     v29 = ireduce.i32 v28
//     v30 = isub v0, v29  ; v0 = 0x0007_0000
//     v31 = iconst.i32 0
//     v32 = icmp eq v3, v31  ; v3 = 0x0003_0000, v31 = 0
//     v33 = iconst.i32 0x7fff_0000
//     v34 = iconst.i32 -2147483648
//     v35 = icmp eq v1, v31  ; v1 = 0x0008_0000, v31 = 0
//     v36 = icmp slt v1, v31  ; v1 = 0x0008_0000, v31 = 0
//     v37 = select v36, v34, v33  ; v34 = -2147483648, v33 = 0x7fff_0000
//     v38 = select v35, v31, v37  ; v31 = 0
//     v39 = iconst.i32 1
//     v40 = select v32, v39, v3  ; v39 = 1, v3 = 0x0003_0000
//     v41 = sextend.i64 v1  ; v1 = 0x0008_0000
//     v42 = iconst.i64 16
//     v43 = ishl v41, v42  ; v42 = 16
//     v44 = sextend.i64 v40
//     v45 = sdiv v43, v44
//     v46 = ireduce.i32 v45
//     v47 = select v32, v38, v46
//     v48 = iconst.i32 16
//     v49 = sshr v47, v48  ; v48 = 16
//     v50 = ishl v49, v48  ; v48 = 16
//     v51 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v52 = sextend.i64 v50
//     v53 = imul v51, v52
//     v54 = iconst.i64 16
//     v55 = sshr v53, v54  ; v54 = 16
//     v56 = ireduce.i32 v55
//     v57 = isub v1, v56  ; v1 = 0x0008_0000
//     v58 = iconst.i32 0
//     v59 = icmp eq v3, v58  ; v3 = 0x0003_0000, v58 = 0
//     v60 = iconst.i32 0x7fff_0000
//     v61 = iconst.i32 -2147483648
//     v62 = icmp eq v2, v58  ; v2 = 0x0009_0000, v58 = 0
//     v63 = icmp slt v2, v58  ; v2 = 0x0009_0000, v58 = 0
//     v64 = select v63, v61, v60  ; v61 = -2147483648, v60 = 0x7fff_0000
//     v65 = select v62, v58, v64  ; v58 = 0
//     v66 = iconst.i32 1
//     v67 = select v59, v66, v3  ; v66 = 1, v3 = 0x0003_0000
//     v68 = sextend.i64 v2  ; v2 = 0x0009_0000
//     v69 = iconst.i64 16
//     v70 = ishl v68, v69  ; v69 = 16
//     v71 = sextend.i64 v67
//     v72 = sdiv v70, v71
//     v73 = ireduce.i32 v72
//     v74 = select v59, v65, v73
//     v75 = iconst.i32 16
//     v76 = sshr v74, v75  ; v75 = 16
//     v77 = ishl v76, v75  ; v75 = 16
//     v78 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v79 = sextend.i64 v77
//     v80 = imul v78, v79
//     v81 = iconst.i64 16
//     v82 = sshr v80, v81  ; v81 = 16
//     v83 = ireduce.i32 v82
//     v84 = isub v2, v83  ; v2 = 0x0009_0000
//     v85 = iadd v30, v57
//     v86 = iadd v85, v84
//     v87 = iconst.i32 0x0002_fd71
//     v88 = icmp sgt v86, v87  ; v87 = 0x0002_fd71
//     v89 = sextend.i32 v88
//     v90 = iconst.i8 1
//     v91 = iconst.i8 0
//     v92 = select v89, v90, v91  ; v90 = 1, v91 = 0
//     v93 = iconst.i32 0x0003_028f
//     v94 = icmp slt v86, v93  ; v93 = 0x0003_028f
//     v95 = sextend.i32 v94
//     v96 = iconst.i8 1
//     v97 = iconst.i8 0
//     v98 = select v95, v96, v97  ; v96 = 1, v97 = 0
//     v99 = iconst.i8 0
//     v100 = iconst.i8 1
//     v101 = icmp ne v92, v99  ; v99 = 0
//     v102 = icmp ne v98, v99  ; v99 = 0
//     v103 = select v102, v100, v99  ; v100 = 1, v99 = 0
//     v104 = select v101, v103, v99  ; v99 = 0
//     return v104
//
// block1:
//     v105 = iconst.i8 0
//     return v105  ; v105 = 0
// }
// run: == true
