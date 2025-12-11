// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 edge0 = vec3(0.0, 0.0, 0.0);
    vec3 edge1 = vec3(10.0, 10.0, 10.0);
    vec3 x = vec3(5.0, 2.5, 7.5);
    vec3 result = smoothstep(edge0, edge1, x);
    // Expected: smoothstep(0,10,5)≈0.5, smoothstep(0,10,2.5)≈0.15625, smoothstep(0,10,7.5)≈0.84375
    // Sum ≈ 1.5
    float sum = result.x + result.y + result.z;
    return sum > 1.45 && sum < 1.55;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0x000a_0000
//     v4 = iconst.i32 0x000a_0000
//     v5 = iconst.i32 0x000a_0000
//     v6 = iconst.i32 0x0005_0000
//     v7 = iconst.i32 0x0002_8000
//     v8 = iconst.i32 0x0007_8000
//     v9 = iconst.i32 0
//     v10 = iconst.i32 0x0001_0000
//     v11 = iconst.i32 0x0002_0000
//     v12 = iconst.i32 0x0003_0000
//     v13 = isub v6, v0  ; v6 = 0x0005_0000, v0 = 0
//     v14 = isub v3, v0  ; v3 = 0x000a_0000, v0 = 0
//     v15 = iconst.i32 0
//     v16 = icmp eq v14, v15  ; v15 = 0
//     v17 = iconst.i32 0x7fff_0000
//     v18 = iconst.i32 -2147483648
//     v19 = icmp eq v13, v15  ; v15 = 0
//     v20 = icmp slt v13, v15  ; v15 = 0
//     v21 = select v20, v18, v17  ; v18 = -2147483648, v17 = 0x7fff_0000
//     v22 = select v19, v15, v21  ; v15 = 0
//     v23 = iconst.i32 1
//     v24 = select v16, v23, v14  ; v23 = 1
//     v25 = sextend.i64 v13
//     v26 = iconst.i64 16
//     v27 = ishl v25, v26  ; v26 = 16
//     v28 = sextend.i64 v24
//     v29 = sdiv v27, v28
//     v30 = ireduce.i32 v29
//     v31 = select v16, v22, v30
//     v32 = icmp sgt v31, v9  ; v9 = 0
//     v33 = select v32, v31, v9  ; v9 = 0
//     v34 = icmp slt v33, v10  ; v10 = 0x0001_0000
//     v35 = select v34, v33, v10  ; v10 = 0x0001_0000
//     v36 = sextend.i64 v35
//     v37 = sextend.i64 v35
//     v38 = imul v36, v37
//     v39 = iconst.i64 16
//     v40 = sshr v38, v39  ; v39 = 16
//     v41 = ireduce.i32 v40
//     v42 = sextend.i64 v11  ; v11 = 0x0002_0000
//     v43 = sextend.i64 v35
//     v44 = imul v42, v43
//     v45 = iconst.i64 16
//     v46 = sshr v44, v45  ; v45 = 16
//     v47 = ireduce.i32 v46
//     v48 = isub v12, v47  ; v12 = 0x0003_0000
//     v49 = sextend.i64 v41
//     v50 = sextend.i64 v48
//     v51 = imul v49, v50
//     v52 = iconst.i64 16
//     v53 = sshr v51, v52  ; v52 = 16
//     v54 = ireduce.i32 v53
//     v55 = isub v7, v1  ; v7 = 0x0002_8000, v1 = 0
//     v56 = isub v4, v1  ; v4 = 0x000a_0000, v1 = 0
//     v57 = iconst.i32 0
//     v58 = icmp eq v56, v57  ; v57 = 0
//     v59 = iconst.i32 0x7fff_0000
//     v60 = iconst.i32 -2147483648
//     v61 = icmp eq v55, v57  ; v57 = 0
//     v62 = icmp slt v55, v57  ; v57 = 0
//     v63 = select v62, v60, v59  ; v60 = -2147483648, v59 = 0x7fff_0000
//     v64 = select v61, v57, v63  ; v57 = 0
//     v65 = iconst.i32 1
//     v66 = select v58, v65, v56  ; v65 = 1
//     v67 = sextend.i64 v55
//     v68 = iconst.i64 16
//     v69 = ishl v67, v68  ; v68 = 16
//     v70 = sextend.i64 v66
//     v71 = sdiv v69, v70
//     v72 = ireduce.i32 v71
//     v73 = select v58, v64, v72
//     v74 = icmp sgt v73, v9  ; v9 = 0
//     v75 = select v74, v73, v9  ; v9 = 0
//     v76 = icmp slt v75, v10  ; v10 = 0x0001_0000
//     v77 = select v76, v75, v10  ; v10 = 0x0001_0000
//     v78 = sextend.i64 v77
//     v79 = sextend.i64 v77
//     v80 = imul v78, v79
//     v81 = iconst.i64 16
//     v82 = sshr v80, v81  ; v81 = 16
//     v83 = ireduce.i32 v82
//     v84 = sextend.i64 v11  ; v11 = 0x0002_0000
//     v85 = sextend.i64 v77
//     v86 = imul v84, v85
//     v87 = iconst.i64 16
//     v88 = sshr v86, v87  ; v87 = 16
//     v89 = ireduce.i32 v88
//     v90 = isub v12, v89  ; v12 = 0x0003_0000
//     v91 = sextend.i64 v83
//     v92 = sextend.i64 v90
//     v93 = imul v91, v92
//     v94 = iconst.i64 16
//     v95 = sshr v93, v94  ; v94 = 16
//     v96 = ireduce.i32 v95
//     v97 = isub v8, v2  ; v8 = 0x0007_8000, v2 = 0
//     v98 = isub v5, v2  ; v5 = 0x000a_0000, v2 = 0
//     v99 = iconst.i32 0
//     v100 = icmp eq v98, v99  ; v99 = 0
//     v101 = iconst.i32 0x7fff_0000
//     v102 = iconst.i32 -2147483648
//     v103 = icmp eq v97, v99  ; v99 = 0
//     v104 = icmp slt v97, v99  ; v99 = 0
//     v105 = select v104, v102, v101  ; v102 = -2147483648, v101 = 0x7fff_0000
//     v106 = select v103, v99, v105  ; v99 = 0
//     v107 = iconst.i32 1
//     v108 = select v100, v107, v98  ; v107 = 1
//     v109 = sextend.i64 v97
//     v110 = iconst.i64 16
//     v111 = ishl v109, v110  ; v110 = 16
//     v112 = sextend.i64 v108
//     v113 = sdiv v111, v112
//     v114 = ireduce.i32 v113
//     v115 = select v100, v106, v114
//     v116 = icmp sgt v115, v9  ; v9 = 0
//     v117 = select v116, v115, v9  ; v9 = 0
//     v118 = icmp slt v117, v10  ; v10 = 0x0001_0000
//     v119 = select v118, v117, v10  ; v10 = 0x0001_0000
//     v120 = sextend.i64 v119
//     v121 = sextend.i64 v119
//     v122 = imul v120, v121
//     v123 = iconst.i64 16
//     v124 = sshr v122, v123  ; v123 = 16
//     v125 = ireduce.i32 v124
//     v126 = sextend.i64 v11  ; v11 = 0x0002_0000
//     v127 = sextend.i64 v119
//     v128 = imul v126, v127
//     v129 = iconst.i64 16
//     v130 = sshr v128, v129  ; v129 = 16
//     v131 = ireduce.i32 v130
//     v132 = isub v12, v131  ; v12 = 0x0003_0000
//     v133 = sextend.i64 v125
//     v134 = sextend.i64 v132
//     v135 = imul v133, v134
//     v136 = iconst.i64 16
//     v137 = sshr v135, v136  ; v136 = 16
//     v138 = ireduce.i32 v137
//     v139 = iadd v54, v96
//     v140 = iadd v139, v138
//     v141 = iconst.i32 0x0001_7333
//     v142 = icmp sgt v140, v141  ; v141 = 0x0001_7333
//     v143 = sextend.i32 v142
//     v144 = iconst.i8 1
//     v145 = iconst.i8 0
//     v146 = select v143, v144, v145  ; v144 = 1, v145 = 0
//     v147 = iconst.i32 0x0001_8ccd
//     v148 = icmp slt v140, v147  ; v147 = 0x0001_8ccd
//     v149 = sextend.i32 v148
//     v150 = iconst.i8 1
//     v151 = iconst.i8 0
//     v152 = select v149, v150, v151  ; v150 = 1, v151 = 0
//     v153 = iconst.i8 0
//     v154 = iconst.i8 1
//     v155 = icmp ne v146, v153  ; v153 = 0
//     v156 = icmp ne v152, v153  ; v153 = 0
//     v157 = select v156, v154, v153  ; v154 = 1, v153 = 0
//     v158 = select v155, v157, v153  ; v153 = 0
//     return v158
//
// block1:
//     v159 = iconst.i8 0
//     return v159  ; v159 = 0
// }
// run: == true
