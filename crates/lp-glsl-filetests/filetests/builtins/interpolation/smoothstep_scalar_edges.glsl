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
//     v55 = iconst.i32 0
//     v56 = iconst.i32 0x000a_0000
//     v57 = iconst.i32 0
//     v58 = iconst.i32 0x0005_0000
//     v59 = iconst.i32 0x000a_0000
//     v60 = iconst.i32 0
//     v61 = iconst.i32 0x0001_0000
//     v62 = iconst.i32 0x0002_0000
//     v63 = iconst.i32 0x0003_0000
//     v64 = isub v57, v55  ; v57 = 0, v55 = 0
//     v65 = isub v56, v55  ; v56 = 0x000a_0000, v55 = 0
//     v66 = sextend.i64 v64
//     v67 = iconst.i64 16
//     v68 = ishl v66, v67  ; v67 = 16
//     v69 = sextend.i64 v65
//     v70 = sdiv v68, v69
//     v71 = ireduce.i32 v70
//     v72 = icmp sge v71, v60  ; v60 = 0
//     v73 = select v72, v71, v60  ; v60 = 0
//     v74 = icmp sle v73, v61  ; v61 = 0x0001_0000
//     v75 = select v74, v73, v61  ; v61 = 0x0001_0000
//     v76 = sextend.i64 v75
//     v77 = sextend.i64 v75
//     v78 = imul v76, v77
//     v79 = iconst.i64 16
//     v80 = sshr v78, v79  ; v79 = 16
//     v81 = ireduce.i32 v80
//     v82 = sextend.i64 v62  ; v62 = 0x0002_0000
//     v83 = sextend.i64 v75
//     v84 = imul v82, v83
//     v85 = iconst.i64 16
//     v86 = sshr v84, v85  ; v85 = 16
//     v87 = ireduce.i32 v86
//     v88 = isub v63, v87  ; v63 = 0x0003_0000
//     v89 = sextend.i64 v81
//     v90 = sextend.i64 v88
//     v91 = imul v89, v90
//     v92 = iconst.i64 16
//     v93 = sshr v91, v92  ; v92 = 16
//     v94 = ireduce.i32 v93
//     v95 = isub v58, v55  ; v58 = 0x0005_0000, v55 = 0
//     v96 = isub v56, v55  ; v56 = 0x000a_0000, v55 = 0
//     v97 = sextend.i64 v95
//     v98 = iconst.i64 16
//     v99 = ishl v97, v98  ; v98 = 16
//     v100 = sextend.i64 v96
//     v101 = sdiv v99, v100
//     v102 = ireduce.i32 v101
//     v103 = icmp sge v102, v60  ; v60 = 0
//     v104 = select v103, v102, v60  ; v60 = 0
//     v105 = icmp sle v104, v61  ; v61 = 0x0001_0000
//     v106 = select v105, v104, v61  ; v61 = 0x0001_0000
//     v107 = sextend.i64 v106
//     v108 = sextend.i64 v106
//     v109 = imul v107, v108
//     v110 = iconst.i64 16
//     v111 = sshr v109, v110  ; v110 = 16
//     v112 = ireduce.i32 v111
//     v113 = sextend.i64 v62  ; v62 = 0x0002_0000
//     v114 = sextend.i64 v106
//     v115 = imul v113, v114
//     v116 = iconst.i64 16
//     v117 = sshr v115, v116  ; v116 = 16
//     v118 = ireduce.i32 v117
//     v119 = isub v63, v118  ; v63 = 0x0003_0000
//     v120 = sextend.i64 v112
//     v121 = sextend.i64 v119
//     v122 = imul v120, v121
//     v123 = iconst.i64 16
//     v124 = sshr v122, v123  ; v123 = 16
//     v125 = ireduce.i32 v124
//     v126 = isub v59, v55  ; v59 = 0x000a_0000, v55 = 0
//     v127 = isub v56, v55  ; v56 = 0x000a_0000, v55 = 0
//     v128 = sextend.i64 v126
//     v129 = iconst.i64 16
//     v130 = ishl v128, v129  ; v129 = 16
//     v131 = sextend.i64 v127
//     v132 = sdiv v130, v131
//     v133 = ireduce.i32 v132
//     v134 = icmp sge v133, v60  ; v60 = 0
//     v135 = select v134, v133, v60  ; v60 = 0
//     v136 = icmp sle v135, v61  ; v61 = 0x0001_0000
//     v137 = select v136, v135, v61  ; v61 = 0x0001_0000
//     v138 = sextend.i64 v137
//     v139 = sextend.i64 v137
//     v140 = imul v138, v139
//     v141 = iconst.i64 16
//     v142 = sshr v140, v141  ; v141 = 16
//     v143 = ireduce.i32 v142
//     v144 = sextend.i64 v62  ; v62 = 0x0002_0000
//     v145 = sextend.i64 v137
//     v146 = imul v144, v145
//     v147 = iconst.i64 16
//     v148 = sshr v146, v147  ; v147 = 16
//     v149 = ireduce.i32 v148
//     v150 = isub v63, v149  ; v63 = 0x0003_0000
//     v151 = sextend.i64 v143
//     v152 = sextend.i64 v150
//     v153 = imul v151, v152
//     v154 = iconst.i64 16
//     v155 = sshr v153, v154  ; v154 = 16
//     v156 = ireduce.i32 v155
//     v157 = iadd v94, v125
//     v158 = iadd v157, v156
//     v159 = iconst.i32 0x0001_7d71
//     v160 = icmp sgt v158, v159  ; v159 = 0x0001_7d71
//     v40 = iconst.i8 1
//     v41 = iconst.i8 0
//     v42 = select v160, v40, v41  ; v40 = 1, v41 = 0
//     v161 = iconst.i32 0x0001_828f
//     v162 = icmp slt v158, v161  ; v161 = 0x0001_828f
//     v45 = iconst.i8 1
//     v46 = iconst.i8 0
//     v47 = select v162, v45, v46  ; v45 = 1, v46 = 0
//     v48 = iconst.i8 0
//     v49 = iconst.i8 1
//     v50 = icmp ne v42, v48  ; v48 = 0
//     v51 = icmp ne v47, v48  ; v48 = 0
//     v52 = select v51, v49, v48  ; v49 = 1, v48 = 0
//     v53 = select v50, v52, v48  ; v48 = 0
//     return v53
//
// block1:
//     v54 = iconst.i8 0
//     return v54  ; v54 = 0
// }
// run: == true
