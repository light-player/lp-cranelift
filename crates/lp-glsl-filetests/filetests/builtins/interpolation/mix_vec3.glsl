// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, vec3(0.5, 0.5, 0.5));  // (5.0, 10.0, 15.0)
    // Validate: x=5, y=10, z=15, sum=30
    float sum = result.x + result.y + result.z;
    return sum > 29.99 && sum < 30.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v43 = iconst.i32 0
//     v44 = iconst.i32 0
//     v45 = iconst.i32 0
//     v46 = iconst.i32 0x000a_0000
//     v47 = iconst.i32 0x0014_0000
//     v48 = iconst.i32 0x001e_0000
//     v49 = iconst.i32 0x8000
//     v50 = iconst.i32 0x8000
//     v51 = iconst.i32 0x8000
//     v52 = iconst.i32 0x0001_0000
//     v53 = isub v52, v49  ; v52 = 0x0001_0000, v49 = 0x8000
//     v54 = sextend.i64 v43  ; v43 = 0
//     v55 = sextend.i64 v53
//     v56 = imul v54, v55
//     v57 = iconst.i64 16
//     v58 = sshr v56, v57  ; v57 = 16
//     v59 = ireduce.i32 v58
//     v60 = sextend.i64 v46  ; v46 = 0x000a_0000
//     v61 = sextend.i64 v49  ; v49 = 0x8000
//     v62 = imul v60, v61
//     v63 = iconst.i64 16
//     v64 = sshr v62, v63  ; v63 = 16
//     v65 = ireduce.i32 v64
//     v66 = iadd v59, v65
//     v67 = iconst.i32 0x0001_0000
//     v68 = isub v67, v50  ; v67 = 0x0001_0000, v50 = 0x8000
//     v69 = sextend.i64 v44  ; v44 = 0
//     v70 = sextend.i64 v68
//     v71 = imul v69, v70
//     v72 = iconst.i64 16
//     v73 = sshr v71, v72  ; v72 = 16
//     v74 = ireduce.i32 v73
//     v75 = sextend.i64 v47  ; v47 = 0x0014_0000
//     v76 = sextend.i64 v50  ; v50 = 0x8000
//     v77 = imul v75, v76
//     v78 = iconst.i64 16
//     v79 = sshr v77, v78  ; v78 = 16
//     v80 = ireduce.i32 v79
//     v81 = iadd v74, v80
//     v82 = iconst.i32 0x0001_0000
//     v83 = isub v82, v51  ; v82 = 0x0001_0000, v51 = 0x8000
//     v84 = sextend.i64 v45  ; v45 = 0
//     v85 = sextend.i64 v83
//     v86 = imul v84, v85
//     v87 = iconst.i64 16
//     v88 = sshr v86, v87  ; v87 = 16
//     v89 = ireduce.i32 v88
//     v90 = sextend.i64 v48  ; v48 = 0x001e_0000
//     v91 = sextend.i64 v51  ; v51 = 0x8000
//     v92 = imul v90, v91
//     v93 = iconst.i64 16
//     v94 = sshr v92, v93  ; v93 = 16
//     v95 = ireduce.i32 v94
//     v96 = iadd v89, v95
//     v97 = iadd v66, v81
//     v98 = iadd v97, v96
//     v99 = iconst.i32 0x001d_fd71
//     v100 = icmp sgt v98, v99  ; v99 = 0x001d_fd71
//     v28 = iconst.i8 1
//     v29 = iconst.i8 0
//     v30 = select v100, v28, v29  ; v28 = 1, v29 = 0
//     v101 = iconst.i32 0x001e_028f
//     v102 = icmp slt v98, v101  ; v101 = 0x001e_028f
//     v33 = iconst.i8 1
//     v34 = iconst.i8 0
//     v35 = select v102, v33, v34  ; v33 = 1, v34 = 0
//     v36 = iconst.i8 0
//     v37 = iconst.i8 1
//     v38 = icmp ne v30, v36  ; v36 = 0
//     v39 = icmp ne v35, v36  ; v36 = 0
//     v40 = select v39, v37, v36  ; v37 = 1, v36 = 0
//     v41 = select v38, v40, v36  ; v36 = 0
//     return v41
//
// block1:
//     v42 = iconst.i8 0
//     return v42  ; v42 = 0
// }
// run: == true
