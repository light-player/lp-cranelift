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
//     v35 = iconst.i32 0x0007_0000
//     v36 = iconst.i32 0x0008_0000
//     v37 = iconst.i32 0x0009_0000
//     v38 = iconst.i32 0x0003_0000
//     v39 = sextend.i64 v35  ; v35 = 0x0007_0000
//     v40 = iconst.i64 16
//     v41 = ishl v39, v40  ; v40 = 16
//     v42 = sextend.i64 v38  ; v38 = 0x0003_0000
//     v43 = sdiv v41, v42
//     v44 = ireduce.i32 v43
//     v45 = iconst.i64 16
//     v46 = sextend.i64 v44
//     v47 = sshr v46, v45  ; v45 = 16
//     v48 = ishl v47, v45  ; v45 = 16
//     v49 = ireduce.i32 v48
//     v50 = sextend.i64 v38  ; v38 = 0x0003_0000
//     v51 = sextend.i64 v49
//     v52 = imul v50, v51
//     v53 = iconst.i64 16
//     v54 = sshr v52, v53  ; v53 = 16
//     v55 = ireduce.i32 v54
//     v56 = isub v35, v55  ; v35 = 0x0007_0000
//     v57 = sextend.i64 v36  ; v36 = 0x0008_0000
//     v58 = iconst.i64 16
//     v59 = ishl v57, v58  ; v58 = 16
//     v60 = sextend.i64 v38  ; v38 = 0x0003_0000
//     v61 = sdiv v59, v60
//     v62 = ireduce.i32 v61
//     v63 = iconst.i64 16
//     v64 = sextend.i64 v62
//     v65 = sshr v64, v63  ; v63 = 16
//     v66 = ishl v65, v63  ; v63 = 16
//     v67 = ireduce.i32 v66
//     v68 = sextend.i64 v38  ; v38 = 0x0003_0000
//     v69 = sextend.i64 v67
//     v70 = imul v68, v69
//     v71 = iconst.i64 16
//     v72 = sshr v70, v71  ; v71 = 16
//     v73 = ireduce.i32 v72
//     v74 = isub v36, v73  ; v36 = 0x0008_0000
//     v75 = sextend.i64 v37  ; v37 = 0x0009_0000
//     v76 = iconst.i64 16
//     v77 = ishl v75, v76  ; v76 = 16
//     v78 = sextend.i64 v38  ; v38 = 0x0003_0000
//     v79 = sdiv v77, v78
//     v80 = ireduce.i32 v79
//     v81 = iconst.i64 16
//     v82 = sextend.i64 v80
//     v83 = sshr v82, v81  ; v81 = 16
//     v84 = ishl v83, v81  ; v81 = 16
//     v85 = ireduce.i32 v84
//     v86 = sextend.i64 v38  ; v38 = 0x0003_0000
//     v87 = sextend.i64 v85
//     v88 = imul v86, v87
//     v89 = iconst.i64 16
//     v90 = sshr v88, v89  ; v89 = 16
//     v91 = ireduce.i32 v90
//     v92 = isub v37, v91  ; v37 = 0x0009_0000
//     v93 = iadd v56, v74
//     v94 = iadd v93, v92
//     v95 = iconst.i32 0x0002_fd71
//     v96 = icmp sgt v94, v95  ; v95 = 0x0002_fd71
//     v20 = iconst.i8 1
//     v21 = iconst.i8 0
//     v22 = select v96, v20, v21  ; v20 = 1, v21 = 0
//     v97 = iconst.i32 0x0003_028f
//     v98 = icmp slt v94, v97  ; v97 = 0x0003_028f
//     v25 = iconst.i8 1
//     v26 = iconst.i8 0
//     v27 = select v98, v25, v26  ; v25 = 1, v26 = 0
//     v28 = iconst.i8 0
//     v29 = iconst.i8 1
//     v30 = icmp ne v22, v28  ; v28 = 0
//     v31 = icmp ne v27, v28  ; v28 = 0
//     v32 = select v31, v29, v28  ; v29 = 1, v28 = 0
//     v33 = select v30, v32, v28  ; v28 = 0
//     return v33
//
// block1:
//     v34 = iconst.i8 0
//     return v34  ; v34 = 0
// }
// run: == true
