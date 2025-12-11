// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec3 result = smoothstep(0.0, 10.0, vec3(0.0, 5.0, 10.0));
    // Expected: smoothstep(0,10,0)=0, smoothstep(0,10,5)=0.5, smoothstep(0,10,10)=1.0
    // Sum = 1.5
    float sum = result.x + result.y + result.z;
    return sum > 1.49 && sum < 1.51;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0x000a_0000
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0x0005_0000
//     v4 = iconst.i32 0x000a_0000
//     v5 = iconst.i32 0
//     v6 = iconst.i32 0x0001_0000
//     v7 = iconst.i32 0x0002_0000
//     v8 = iconst.i32 0x0003_0000
//     v9 = isub v2, v0  ; v2 = 0, v0 = 0
//     v10 = isub v1, v0  ; v1 = 0x000a_0000, v0 = 0
//     v11 = iconst.i32 0
//     v12 = icmp eq v10, v11  ; v11 = 0
//     v13 = iconst.i32 0x7fff_0000
//     v14 = iconst.i32 -2147483648
//     v15 = icmp eq v9, v11  ; v11 = 0
//     v16 = icmp slt v9, v11  ; v11 = 0
//     v17 = select v16, v14, v13  ; v14 = -2147483648, v13 = 0x7fff_0000
//     v18 = select v15, v11, v17  ; v11 = 0
//     v19 = iconst.i32 1
//     v20 = select v12, v19, v10  ; v19 = 1
//     v21 = sextend.i64 v9
//     v22 = iconst.i64 16
//     v23 = ishl v21, v22  ; v22 = 16
//     v24 = sextend.i64 v20
//     v25 = sdiv v23, v24
//     v26 = ireduce.i32 v25
//     v27 = select v12, v18, v26
//     v28 = icmp sgt v27, v5  ; v5 = 0
//     v29 = select v28, v27, v5  ; v5 = 0
//     v30 = icmp slt v29, v6  ; v6 = 0x0001_0000
//     v31 = select v30, v29, v6  ; v6 = 0x0001_0000
//     v32 = sextend.i64 v31
//     v33 = sextend.i64 v31
//     v34 = imul v32, v33
//     v35 = iconst.i64 16
//     v36 = sshr v34, v35  ; v35 = 16
//     v37 = ireduce.i32 v36
//     v38 = sextend.i64 v7  ; v7 = 0x0002_0000
//     v39 = sextend.i64 v31
//     v40 = imul v38, v39
//     v41 = iconst.i64 16
//     v42 = sshr v40, v41  ; v41 = 16
//     v43 = ireduce.i32 v42
//     v44 = isub v8, v43  ; v8 = 0x0003_0000
//     v45 = sextend.i64 v37
//     v46 = sextend.i64 v44
//     v47 = imul v45, v46
//     v48 = iconst.i64 16
//     v49 = sshr v47, v48  ; v48 = 16
//     v50 = ireduce.i32 v49
//     v51 = isub v3, v0  ; v3 = 0x0005_0000, v0 = 0
//     v52 = isub v1, v0  ; v1 = 0x000a_0000, v0 = 0
//     v53 = iconst.i32 0
//     v54 = icmp eq v52, v53  ; v53 = 0
//     v55 = iconst.i32 0x7fff_0000
//     v56 = iconst.i32 -2147483648
//     v57 = icmp eq v51, v53  ; v53 = 0
//     v58 = icmp slt v51, v53  ; v53 = 0
//     v59 = select v58, v56, v55  ; v56 = -2147483648, v55 = 0x7fff_0000
//     v60 = select v57, v53, v59  ; v53 = 0
//     v61 = iconst.i32 1
//     v62 = select v54, v61, v52  ; v61 = 1
//     v63 = sextend.i64 v51
//     v64 = iconst.i64 16
//     v65 = ishl v63, v64  ; v64 = 16
//     v66 = sextend.i64 v62
//     v67 = sdiv v65, v66
//     v68 = ireduce.i32 v67
//     v69 = select v54, v60, v68
//     v70 = icmp sgt v69, v5  ; v5 = 0
//     v71 = select v70, v69, v5  ; v5 = 0
//     v72 = icmp slt v71, v6  ; v6 = 0x0001_0000
//     v73 = select v72, v71, v6  ; v6 = 0x0001_0000
//     v74 = sextend.i64 v73
//     v75 = sextend.i64 v73
//     v76 = imul v74, v75
//     v77 = iconst.i64 16
//     v78 = sshr v76, v77  ; v77 = 16
//     v79 = ireduce.i32 v78
//     v80 = sextend.i64 v7  ; v7 = 0x0002_0000
//     v81 = sextend.i64 v73
//     v82 = imul v80, v81
//     v83 = iconst.i64 16
//     v84 = sshr v82, v83  ; v83 = 16
//     v85 = ireduce.i32 v84
//     v86 = isub v8, v85  ; v8 = 0x0003_0000
//     v87 = sextend.i64 v79
//     v88 = sextend.i64 v86
//     v89 = imul v87, v88
//     v90 = iconst.i64 16
//     v91 = sshr v89, v90  ; v90 = 16
//     v92 = ireduce.i32 v91
//     v93 = isub v4, v0  ; v4 = 0x000a_0000, v0 = 0
//     v94 = isub v1, v0  ; v1 = 0x000a_0000, v0 = 0
//     v95 = iconst.i32 0
//     v96 = icmp eq v94, v95  ; v95 = 0
//     v97 = iconst.i32 0x7fff_0000
//     v98 = iconst.i32 -2147483648
//     v99 = icmp eq v93, v95  ; v95 = 0
//     v100 = icmp slt v93, v95  ; v95 = 0
//     v101 = select v100, v98, v97  ; v98 = -2147483648, v97 = 0x7fff_0000
//     v102 = select v99, v95, v101  ; v95 = 0
//     v103 = iconst.i32 1
//     v104 = select v96, v103, v94  ; v103 = 1
//     v105 = sextend.i64 v93
//     v106 = iconst.i64 16
//     v107 = ishl v105, v106  ; v106 = 16
//     v108 = sextend.i64 v104
//     v109 = sdiv v107, v108
//     v110 = ireduce.i32 v109
//     v111 = select v96, v102, v110
//     v112 = icmp sgt v111, v5  ; v5 = 0
//     v113 = select v112, v111, v5  ; v5 = 0
//     v114 = icmp slt v113, v6  ; v6 = 0x0001_0000
//     v115 = select v114, v113, v6  ; v6 = 0x0001_0000
//     v116 = sextend.i64 v115
//     v117 = sextend.i64 v115
//     v118 = imul v116, v117
//     v119 = iconst.i64 16
//     v120 = sshr v118, v119  ; v119 = 16
//     v121 = ireduce.i32 v120
//     v122 = sextend.i64 v7  ; v7 = 0x0002_0000
//     v123 = sextend.i64 v115
//     v124 = imul v122, v123
//     v125 = iconst.i64 16
//     v126 = sshr v124, v125  ; v125 = 16
//     v127 = ireduce.i32 v126
//     v128 = isub v8, v127  ; v8 = 0x0003_0000
//     v129 = sextend.i64 v121
//     v130 = sextend.i64 v128
//     v131 = imul v129, v130
//     v132 = iconst.i64 16
//     v133 = sshr v131, v132  ; v132 = 16
//     v134 = ireduce.i32 v133
//     v135 = iadd v50, v92
//     v136 = iadd v135, v134
//     v137 = iconst.i32 0x0001_7d71
//     v138 = icmp sgt v136, v137  ; v137 = 0x0001_7d71
//     v139 = sextend.i32 v138
//     v140 = iconst.i8 1
//     v141 = iconst.i8 0
//     v142 = select v139, v140, v141  ; v140 = 1, v141 = 0
//     v143 = iconst.i32 0x0001_828f
//     v144 = icmp slt v136, v143  ; v143 = 0x0001_828f
//     v145 = sextend.i32 v144
//     v146 = iconst.i8 1
//     v147 = iconst.i8 0
//     v148 = select v145, v146, v147  ; v146 = 1, v147 = 0
//     v149 = iconst.i8 0
//     v150 = iconst.i8 1
//     v151 = icmp ne v142, v149  ; v149 = 0
//     v152 = icmp ne v148, v149  ; v149 = 0
//     v153 = select v152, v150, v149  ; v150 = 1, v149 = 0
//     v154 = select v151, v153, v149  ; v149 = 0
//     return v154
//
// block1:
//     v155 = iconst.i8 0
//     return v155  ; v155 = 0
// }
// run: == true
