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
//     v59 = iconst.i32 0
//     v60 = iconst.i32 0
//     v61 = iconst.i32 0
//     v62 = iconst.i32 0x000a_0000
//     v63 = iconst.i32 0x000a_0000
//     v64 = iconst.i32 0x000a_0000
//     v65 = iconst.i32 0x0005_0000
//     v66 = iconst.i32 0x0002_8000
//     v67 = iconst.i32 0x0007_8000
//     v68 = iconst.i32 0
//     v69 = iconst.i32 0x0001_0000
//     v70 = iconst.i32 0x0002_0000
//     v71 = iconst.i32 0x0003_0000
//     v72 = isub v65, v59  ; v65 = 0x0005_0000, v59 = 0
//     v73 = isub v62, v59  ; v62 = 0x000a_0000, v59 = 0
//     v74 = sextend.i64 v72
//     v75 = iconst.i64 16
//     v76 = ishl v74, v75  ; v75 = 16
//     v77 = sextend.i64 v73
//     v78 = sdiv v76, v77
//     v79 = ireduce.i32 v78
//     v80 = icmp sge v79, v68  ; v68 = 0
//     v81 = select v80, v79, v68  ; v68 = 0
//     v82 = icmp sle v81, v69  ; v69 = 0x0001_0000
//     v83 = select v82, v81, v69  ; v69 = 0x0001_0000
//     v84 = sextend.i64 v83
//     v85 = sextend.i64 v83
//     v86 = imul v84, v85
//     v87 = iconst.i64 16
//     v88 = sshr v86, v87  ; v87 = 16
//     v89 = ireduce.i32 v88
//     v90 = sextend.i64 v70  ; v70 = 0x0002_0000
//     v91 = sextend.i64 v83
//     v92 = imul v90, v91
//     v93 = iconst.i64 16
//     v94 = sshr v92, v93  ; v93 = 16
//     v95 = ireduce.i32 v94
//     v96 = isub v71, v95  ; v71 = 0x0003_0000
//     v97 = sextend.i64 v89
//     v98 = sextend.i64 v96
//     v99 = imul v97, v98
//     v100 = iconst.i64 16
//     v101 = sshr v99, v100  ; v100 = 16
//     v102 = ireduce.i32 v101
//     v103 = isub v66, v60  ; v66 = 0x0002_8000, v60 = 0
//     v104 = isub v63, v60  ; v63 = 0x000a_0000, v60 = 0
//     v105 = sextend.i64 v103
//     v106 = iconst.i64 16
//     v107 = ishl v105, v106  ; v106 = 16
//     v108 = sextend.i64 v104
//     v109 = sdiv v107, v108
//     v110 = ireduce.i32 v109
//     v111 = icmp sge v110, v68  ; v68 = 0
//     v112 = select v111, v110, v68  ; v68 = 0
//     v113 = icmp sle v112, v69  ; v69 = 0x0001_0000
//     v114 = select v113, v112, v69  ; v69 = 0x0001_0000
//     v115 = sextend.i64 v114
//     v116 = sextend.i64 v114
//     v117 = imul v115, v116
//     v118 = iconst.i64 16
//     v119 = sshr v117, v118  ; v118 = 16
//     v120 = ireduce.i32 v119
//     v121 = sextend.i64 v70  ; v70 = 0x0002_0000
//     v122 = sextend.i64 v114
//     v123 = imul v121, v122
//     v124 = iconst.i64 16
//     v125 = sshr v123, v124  ; v124 = 16
//     v126 = ireduce.i32 v125
//     v127 = isub v71, v126  ; v71 = 0x0003_0000
//     v128 = sextend.i64 v120
//     v129 = sextend.i64 v127
//     v130 = imul v128, v129
//     v131 = iconst.i64 16
//     v132 = sshr v130, v131  ; v131 = 16
//     v133 = ireduce.i32 v132
//     v134 = isub v67, v61  ; v67 = 0x0007_8000, v61 = 0
//     v135 = isub v64, v61  ; v64 = 0x000a_0000, v61 = 0
//     v136 = sextend.i64 v134
//     v137 = iconst.i64 16
//     v138 = ishl v136, v137  ; v137 = 16
//     v139 = sextend.i64 v135
//     v140 = sdiv v138, v139
//     v141 = ireduce.i32 v140
//     v142 = icmp sge v141, v68  ; v68 = 0
//     v143 = select v142, v141, v68  ; v68 = 0
//     v144 = icmp sle v143, v69  ; v69 = 0x0001_0000
//     v145 = select v144, v143, v69  ; v69 = 0x0001_0000
//     v146 = sextend.i64 v145
//     v147 = sextend.i64 v145
//     v148 = imul v146, v147
//     v149 = iconst.i64 16
//     v150 = sshr v148, v149  ; v149 = 16
//     v151 = ireduce.i32 v150
//     v152 = sextend.i64 v70  ; v70 = 0x0002_0000
//     v153 = sextend.i64 v145
//     v154 = imul v152, v153
//     v155 = iconst.i64 16
//     v156 = sshr v154, v155  ; v155 = 16
//     v157 = ireduce.i32 v156
//     v158 = isub v71, v157  ; v71 = 0x0003_0000
//     v159 = sextend.i64 v151
//     v160 = sextend.i64 v158
//     v161 = imul v159, v160
//     v162 = iconst.i64 16
//     v163 = sshr v161, v162  ; v162 = 16
//     v164 = ireduce.i32 v163
//     v165 = iadd v102, v133
//     v166 = iadd v165, v164
//     v167 = iconst.i32 0x0001_7333
//     v168 = icmp sgt v166, v167  ; v167 = 0x0001_7333
//     v44 = iconst.i8 1
//     v45 = iconst.i8 0
//     v46 = select v168, v44, v45  ; v44 = 1, v45 = 0
//     v169 = iconst.i32 0x0001_8ccd
//     v170 = icmp slt v166, v169  ; v169 = 0x0001_8ccd
//     v49 = iconst.i8 1
//     v50 = iconst.i8 0
//     v51 = select v170, v49, v50  ; v49 = 1, v50 = 0
//     v52 = iconst.i8 0
//     v53 = iconst.i8 1
//     v54 = icmp ne v46, v52  ; v52 = 0
//     v55 = icmp ne v51, v52  ; v52 = 0
//     v56 = select v55, v53, v52  ; v53 = 1, v52 = 0
//     v57 = select v54, v56, v52  ; v52 = 0
//     return v57
//
// block1:
//     v58 = iconst.i8 0
//     return v58  ; v58 = 0
// }
// run: == true
