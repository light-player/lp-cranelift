// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, 0.25);  // 0*0.75 + vec3(10,20,30)*0.25 = (2.5, 5.0, 7.5)
    // Validate: sum = 2.5 + 5.0 + 7.5 = 15.0
    float sum = result.x + result.y + result.z;
    return sum > 14.99 && sum < 15.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v37 = iconst.i32 0
//     v38 = iconst.i32 0
//     v39 = iconst.i32 0
//     v40 = iconst.i32 0x000a_0000
//     v41 = iconst.i32 0x0014_0000
//     v42 = iconst.i32 0x001e_0000
//     v43 = iconst.i32 0x4000
//     v44 = iconst.i32 0x0001_0000
//     v45 = isub v44, v43  ; v44 = 0x0001_0000, v43 = 0x4000
//     v46 = sextend.i64 v37  ; v37 = 0
//     v47 = sextend.i64 v45
//     v48 = imul v46, v47
//     v49 = iconst.i64 16
//     v50 = sshr v48, v49  ; v49 = 16
//     v51 = ireduce.i32 v50
//     v52 = sextend.i64 v40  ; v40 = 0x000a_0000
//     v53 = sextend.i64 v43  ; v43 = 0x4000
//     v54 = imul v52, v53
//     v55 = iconst.i64 16
//     v56 = sshr v54, v55  ; v55 = 16
//     v57 = ireduce.i32 v56
//     v58 = iadd v51, v57
//     v59 = sextend.i64 v38  ; v38 = 0
//     v60 = sextend.i64 v45
//     v61 = imul v59, v60
//     v62 = iconst.i64 16
//     v63 = sshr v61, v62  ; v62 = 16
//     v64 = ireduce.i32 v63
//     v65 = sextend.i64 v41  ; v41 = 0x0014_0000
//     v66 = sextend.i64 v43  ; v43 = 0x4000
//     v67 = imul v65, v66
//     v68 = iconst.i64 16
//     v69 = sshr v67, v68  ; v68 = 16
//     v70 = ireduce.i32 v69
//     v71 = iadd v64, v70
//     v72 = sextend.i64 v39  ; v39 = 0
//     v73 = sextend.i64 v45
//     v74 = imul v72, v73
//     v75 = iconst.i64 16
//     v76 = sshr v74, v75  ; v75 = 16
//     v77 = ireduce.i32 v76
//     v78 = sextend.i64 v42  ; v42 = 0x001e_0000
//     v79 = sextend.i64 v43  ; v43 = 0x4000
//     v80 = imul v78, v79
//     v81 = iconst.i64 16
//     v82 = sshr v80, v81  ; v81 = 16
//     v83 = ireduce.i32 v82
//     v84 = iadd v77, v83
//     v85 = iadd v58, v71
//     v86 = iadd v85, v84
//     v87 = iconst.i32 0x000e_fd71
//     v88 = icmp sgt v86, v87  ; v87 = 0x000e_fd71
//     v22 = iconst.i8 1
//     v23 = iconst.i8 0
//     v24 = select v88, v22, v23  ; v22 = 1, v23 = 0
//     v89 = iconst.i32 0x000f_028f
//     v90 = icmp slt v86, v89  ; v89 = 0x000f_028f
//     v27 = iconst.i8 1
//     v28 = iconst.i8 0
//     v29 = select v90, v27, v28  ; v27 = 1, v28 = 0
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
