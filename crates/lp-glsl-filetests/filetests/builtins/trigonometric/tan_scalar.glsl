// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    float pi_4 = 0.785398163;  // π/4
    return tan(pi_4);
}

// function u0:0() -> i32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = %tanf sig0
//
// block0:
//     v3 = iconst.i32 0xc910
//     v4 = iconst.i32 0x0003_243f
//     v5 = iconst.i32 0x0001_921f
//     v6 = iconst.i32 0x0006_487e
//     v7 = iconst.i32 0
//     v8 = iadd v4, v5  ; v4 = 0x0003_243f, v5 = 0x0001_921f
//     v9 = iconst.i32 1
//     v10 = iconst.i32 2
//     v11 = iconst.i32 3
//     v12 = icmp sle v3, v5  ; v3 = 0xc910, v5 = 0x0001_921f
//     v13 = icmp sle v3, v4  ; v3 = 0xc910, v4 = 0x0003_243f
//     v14 = icmp sle v3, v8  ; v3 = 0xc910
//     v15 = icmp sgt v3, v8  ; v3 = 0xc910
//     v16 = isub v4, v3  ; v4 = 0x0003_243f, v3 = 0xc910
//     v17 = isub v3, v4  ; v3 = 0xc910, v4 = 0x0003_243f
//     v18 = isub v6, v3  ; v6 = 0x0006_487e, v3 = 0xc910
//     v19 = select v13, v16, v3  ; v3 = 0xc910
//     v20 = select v14, v17, v19
//     v21 = select v15, v18, v20
//     v22 = bnot v12
//     v23 = band v13, v22
//     v24 = select v12, v7, v7  ; v7 = 0, v7 = 0
//     v25 = select v23, v9, v24  ; v9 = 1
//     v26 = bnot v13
//     v27 = band v14, v26
//     v28 = select v27, v10, v25  ; v10 = 2
//     v29 = select v15, v11, v28  ; v11 = 3
//     v30 = iconst.i32 0x9b74
//     v31 = iconst.i32 0
//     v32 = iconst.i32 0
//     v33 = iconst.i32 -1
//     v34 = iconst.i32 1
//     v35 = icmp slt v21, v32  ; v32 = 0
//     v36 = select v35, v33, v34  ; v33 = -1, v34 = 1
//     v37 = iconst.i32 0
//     v38 = sshr v31, v37  ; v31 = 0, v37 = 0
//     v39 = imul v36, v38
//     v40 = isub v30, v39  ; v30 = 0x9b74
//     v41 = sshr v30, v37  ; v30 = 0x9b74, v37 = 0
//     v42 = imul v36, v41
//     v43 = iadd v31, v42  ; v31 = 0
//     v44 = iconst.i32 0xc90f
//     v45 = imul v36, v44  ; v44 = 0xc90f
//     v46 = isub v21, v45
//     v47 = icmp slt v46, v32  ; v32 = 0
//     v48 = select v47, v33, v34  ; v33 = -1, v34 = 1
//     v49 = iconst.i32 1
//     v50 = sshr v43, v49  ; v49 = 1
//     v51 = imul v48, v50
//     v52 = isub v40, v51
//     v53 = sshr v40, v49  ; v49 = 1
//     v54 = imul v48, v53
//     v55 = iadd v43, v54
//     v56 = iconst.i32 0x76b1
//     v57 = imul v48, v56  ; v56 = 0x76b1
//     v58 = isub v46, v57
//     v59 = icmp slt v58, v32  ; v32 = 0
//     v60 = select v59, v33, v34  ; v33 = -1, v34 = 1
//     v61 = iconst.i32 2
//     v62 = sshr v55, v61  ; v61 = 2
//     v63 = imul v60, v62
//     v64 = isub v52, v63
//     v65 = sshr v52, v61  ; v61 = 2
//     v66 = imul v60, v65
//     v67 = iadd v55, v66
//     v68 = iconst.i32 0x3eb6
//     v69 = imul v60, v68  ; v68 = 0x3eb6
//     v70 = isub v58, v69
//     v71 = icmp slt v70, v32  ; v32 = 0
//     v72 = select v71, v33, v34  ; v33 = -1, v34 = 1
//     v73 = iconst.i32 3
//     v74 = sshr v67, v73  ; v73 = 3
//     v75 = imul v72, v74
//     v76 = isub v64, v75
//     v77 = sshr v64, v73  ; v73 = 3
//     v78 = imul v72, v77
//     v79 = iadd v67, v78
//     v80 = iconst.i32 8149
//     v81 = imul v72, v80  ; v80 = 8149
//     v82 = isub v70, v81
//     v83 = icmp slt v82, v32  ; v32 = 0
//     v84 = select v83, v33, v34  ; v33 = -1, v34 = 1
//     v85 = iconst.i32 4
//     v86 = sshr v79, v85  ; v85 = 4
//     v87 = imul v84, v86
//     v88 = isub v76, v87
//     v89 = sshr v76, v85  ; v85 = 4
//     v90 = imul v84, v89
//     v91 = iadd v79, v90
//     v92 = iconst.i32 4090
//     v93 = imul v84, v92  ; v92 = 4090
//     v94 = isub v82, v93
//     v95 = icmp slt v94, v32  ; v32 = 0
//     v96 = select v95, v33, v34  ; v33 = -1, v34 = 1
//     v97 = iconst.i32 5
//     v98 = sshr v91, v97  ; v97 = 5
//     v99 = imul v96, v98
//     v100 = isub v88, v99
//     v101 = sshr v88, v97  ; v97 = 5
//     v102 = imul v96, v101
//     v103 = iadd v91, v102
//     v104 = iconst.i32 2047
//     v105 = imul v96, v104  ; v104 = 2047
//     v106 = isub v94, v105
//     v107 = icmp slt v106, v32  ; v32 = 0
//     v108 = select v107, v33, v34  ; v33 = -1, v34 = 1
//     v109 = iconst.i32 6
//     v110 = sshr v103, v109  ; v109 = 6
//     v111 = imul v108, v110
//     v112 = isub v100, v111
//     v113 = sshr v100, v109  ; v109 = 6
//     v114 = imul v108, v113
//     v115 = iadd v103, v114
//     v116 = iconst.i32 1023
//     v117 = imul v108, v116  ; v116 = 1023
//     v118 = isub v106, v117
//     v119 = icmp slt v118, v32  ; v32 = 0
//     v120 = select v119, v33, v34  ; v33 = -1, v34 = 1
//     v121 = iconst.i32 7
//     v122 = sshr v115, v121  ; v121 = 7
//     v123 = imul v120, v122
//     v124 = isub v112, v123
//     v125 = sshr v112, v121  ; v121 = 7
//     v126 = imul v120, v125
//     v127 = iadd v115, v126
//     v128 = iconst.i32 511
//     v129 = imul v120, v128  ; v128 = 511
//     v130 = isub v118, v129
//     v131 = icmp slt v130, v32  ; v32 = 0
//     v132 = select v131, v33, v34  ; v33 = -1, v34 = 1
//     v133 = iconst.i32 8
//     v134 = sshr v127, v133  ; v133 = 8
//     v135 = imul v132, v134
//     v136 = isub v124, v135
//     v137 = sshr v124, v133  ; v133 = 8
//     v138 = imul v132, v137
//     v139 = iadd v127, v138
//     v140 = iconst.i32 255
//     v141 = imul v132, v140  ; v140 = 255
//     v142 = isub v130, v141
//     v143 = icmp slt v142, v32  ; v32 = 0
//     v144 = select v143, v33, v34  ; v33 = -1, v34 = 1
//     v145 = iconst.i32 9
//     v146 = sshr v139, v145  ; v145 = 9
//     v147 = imul v144, v146
//     v148 = isub v136, v147
//     v149 = sshr v136, v145  ; v145 = 9
//     v150 = imul v144, v149
//     v151 = iadd v139, v150
//     v152 = iconst.i32 127
//     v153 = imul v144, v152  ; v152 = 127
//     v154 = isub v142, v153
//     v155 = icmp slt v154, v32  ; v32 = 0
//     v156 = select v155, v33, v34  ; v33 = -1, v34 = 1
//     v157 = iconst.i32 10
//     v158 = sshr v151, v157  ; v157 = 10
//     v159 = imul v156, v158
//     v160 = isub v148, v159
//     v161 = sshr v148, v157  ; v157 = 10
//     v162 = imul v156, v161
//     v163 = iadd v151, v162
//     v164 = iconst.i32 63
//     v165 = imul v156, v164  ; v164 = 63
//     v166 = isub v154, v165
//     v167 = icmp slt v166, v32  ; v32 = 0
//     v168 = select v167, v33, v34  ; v33 = -1, v34 = 1
//     v169 = iconst.i32 11
//     v170 = sshr v163, v169  ; v169 = 11
//     v171 = imul v168, v170
//     v172 = isub v160, v171
//     v173 = sshr v160, v169  ; v169 = 11
//     v174 = imul v168, v173
//     v175 = iadd v163, v174
//     v176 = iconst.i32 31
//     v177 = imul v168, v176  ; v176 = 31
//     v178 = isub v166, v177
//     v179 = icmp slt v178, v32  ; v32 = 0
//     v180 = select v179, v33, v34  ; v33 = -1, v34 = 1
//     v181 = iconst.i32 12
//     v182 = sshr v175, v181  ; v181 = 12
//     v183 = imul v180, v182
//     v184 = isub v172, v183
//     v185 = sshr v172, v181  ; v181 = 12
//     v186 = imul v180, v185
//     v187 = iadd v175, v186
//     v188 = iconst.i32 15
//     v189 = imul v180, v188  ; v188 = 15
//     v190 = isub v178, v189
//     v191 = icmp slt v190, v32  ; v32 = 0
//     v192 = select v191, v33, v34  ; v33 = -1, v34 = 1
//     v193 = iconst.i32 13
//     v194 = sshr v187, v193  ; v193 = 13
//     v195 = imul v192, v194
//     v196 = isub v184, v195
//     v197 = sshr v184, v193  ; v193 = 13
//     v198 = imul v192, v197
//     v199 = iadd v187, v198
//     v200 = iconst.i32 7
//     v201 = imul v192, v200  ; v200 = 7
//     v202 = isub v190, v201
//     v203 = icmp slt v202, v32  ; v32 = 0
//     v204 = select v203, v33, v34  ; v33 = -1, v34 = 1
//     v205 = iconst.i32 14
//     v206 = sshr v199, v205  ; v205 = 14
//     v207 = imul v204, v206
//     v208 = isub v196, v207
//     v209 = sshr v196, v205  ; v205 = 14
//     v210 = imul v204, v209
//     v211 = iadd v199, v210
//     v212 = iconst.i32 3
//     v213 = imul v204, v212  ; v212 = 3
//     v214 = isub v202, v213
//     v215 = icmp slt v214, v32  ; v32 = 0
//     v216 = select v215, v33, v34  ; v33 = -1, v34 = 1
//     v217 = iconst.i32 15
//     v218 = sshr v211, v217  ; v217 = 15
//     v219 = imul v216, v218
//     v220 = isub v208, v219
//     v221 = sshr v208, v217  ; v217 = 15
//     v222 = imul v216, v221
//     v223 = iadd v211, v222
//     v224 = iconst.i32 1
//     v225 = imul v216, v224  ; v224 = 1
//     v226 = isub v214, v225
//     v227 = iconst.i32 0x0001_a592
//     v228 = sextend.i64 v223
//     v229 = sextend.i64 v227  ; v227 = 0x0001_a592
//     v230 = imul v228, v229
//     v231 = iconst.i64 16
//     v232 = sshr v230, v231  ; v231 = 16
//     v233 = ireduce.i32 v232
//     v234 = sextend.i64 v220
//     v235 = imul v234, v229
//     v236 = sshr v235, v231  ; v231 = 16
//     v237 = ireduce.i32 v236
//     v238 = iconst.i32 0
//     v239 = iconst.i32 1
//     v240 = iconst.i32 2
//     v241 = iconst.i32 3
//     v242 = icmp eq v29, v238  ; v238 = 0
//     v243 = icmp eq v29, v239  ; v239 = 1
//     v244 = icmp eq v29, v240  ; v240 = 2
//     v245 = icmp eq v29, v241  ; v241 = 3
//     v246 = ineg v233
//     v247 = select v244, v246, v233
//     v248 = select v245, v246, v247
//     v249 = select v243, v233, v248
//     v250 = select v242, v233, v249
//     v251 = iconst.i32 0x0003_243f
//     v252 = iconst.i32 0x0001_921f
//     v253 = iconst.i32 0x0006_487e
//     v254 = iconst.i32 0
//     v255 = iadd v251, v252  ; v251 = 0x0003_243f, v252 = 0x0001_921f
//     v256 = iconst.i32 1
//     v257 = iconst.i32 2
//     v258 = iconst.i32 3
//     v259 = icmp sle v3, v252  ; v3 = 0xc910, v252 = 0x0001_921f
//     v260 = icmp sle v3, v251  ; v3 = 0xc910, v251 = 0x0003_243f
//     v261 = icmp sle v3, v255  ; v3 = 0xc910
//     v262 = icmp sgt v3, v255  ; v3 = 0xc910
//     v263 = isub v251, v3  ; v251 = 0x0003_243f, v3 = 0xc910
//     v264 = isub v3, v251  ; v3 = 0xc910, v251 = 0x0003_243f
//     v265 = isub v253, v3  ; v253 = 0x0006_487e, v3 = 0xc910
//     v266 = select v260, v263, v3  ; v3 = 0xc910
//     v267 = select v261, v264, v266
//     v268 = select v262, v265, v267
//     v269 = bnot v259
//     v270 = band v260, v269
//     v271 = select v259, v254, v254  ; v254 = 0, v254 = 0
//     v272 = select v270, v256, v271  ; v256 = 1
//     v273 = bnot v260
//     v274 = band v261, v273
//     v275 = select v274, v257, v272  ; v257 = 2
//     v276 = select v262, v258, v275  ; v258 = 3
//     v277 = iconst.i32 0x9b74
//     v278 = iconst.i32 0
//     v279 = iconst.i32 0
//     v280 = iconst.i32 -1
//     v281 = iconst.i32 1
//     v282 = icmp slt v268, v279  ; v279 = 0
//     v283 = select v282, v280, v281  ; v280 = -1, v281 = 1
//     v284 = iconst.i32 0
//     v285 = sshr v278, v284  ; v278 = 0, v284 = 0
//     v286 = imul v283, v285
//     v287 = isub v277, v286  ; v277 = 0x9b74
//     v288 = sshr v277, v284  ; v277 = 0x9b74, v284 = 0
//     v289 = imul v283, v288
//     v290 = iadd v278, v289  ; v278 = 0
//     v291 = iconst.i32 0xc90f
//     v292 = imul v283, v291  ; v291 = 0xc90f
//     v293 = isub v268, v292
//     v294 = icmp slt v293, v279  ; v279 = 0
//     v295 = select v294, v280, v281  ; v280 = -1, v281 = 1
//     v296 = iconst.i32 1
//     v297 = sshr v290, v296  ; v296 = 1
//     v298 = imul v295, v297
//     v299 = isub v287, v298
//     v300 = sshr v287, v296  ; v296 = 1
//     v301 = imul v295, v300
//     v302 = iadd v290, v301
//     v303 = iconst.i32 0x76b1
//     v304 = imul v295, v303  ; v303 = 0x76b1
//     v305 = isub v293, v304
//     v306 = icmp slt v305, v279  ; v279 = 0
//     v307 = select v306, v280, v281  ; v280 = -1, v281 = 1
//     v308 = iconst.i32 2
//     v309 = sshr v302, v308  ; v308 = 2
//     v310 = imul v307, v309
//     v311 = isub v299, v310
//     v312 = sshr v299, v308  ; v308 = 2
//     v313 = imul v307, v312
//     v314 = iadd v302, v313
//     v315 = iconst.i32 0x3eb6
//     v316 = imul v307, v315  ; v315 = 0x3eb6
//     v317 = isub v305, v316
//     v318 = icmp slt v317, v279  ; v279 = 0
//     v319 = select v318, v280, v281  ; v280 = -1, v281 = 1
//     v320 = iconst.i32 3
//     v321 = sshr v314, v320  ; v320 = 3
//     v322 = imul v319, v321
//     v323 = isub v311, v322
//     v324 = sshr v311, v320  ; v320 = 3
//     v325 = imul v319, v324
//     v326 = iadd v314, v325
//     v327 = iconst.i32 8149
//     v328 = imul v319, v327  ; v327 = 8149
//     v329 = isub v317, v328
//     v330 = icmp slt v329, v279  ; v279 = 0
//     v331 = select v330, v280, v281  ; v280 = -1, v281 = 1
//     v332 = iconst.i32 4
//     v333 = sshr v326, v332  ; v332 = 4
//     v334 = imul v331, v333
//     v335 = isub v323, v334
//     v336 = sshr v323, v332  ; v332 = 4
//     v337 = imul v331, v336
//     v338 = iadd v326, v337
//     v339 = iconst.i32 4090
//     v340 = imul v331, v339  ; v339 = 4090
//     v341 = isub v329, v340
//     v342 = icmp slt v341, v279  ; v279 = 0
//     v343 = select v342, v280, v281  ; v280 = -1, v281 = 1
//     v344 = iconst.i32 5
//     v345 = sshr v338, v344  ; v344 = 5
//     v346 = imul v343, v345
//     v347 = isub v335, v346
//     v348 = sshr v335, v344  ; v344 = 5
//     v349 = imul v343, v348
//     v350 = iadd v338, v349
//     v351 = iconst.i32 2047
//     v352 = imul v343, v351  ; v351 = 2047
//     v353 = isub v341, v352
//     v354 = icmp slt v353, v279  ; v279 = 0
//     v355 = select v354, v280, v281  ; v280 = -1, v281 = 1
//     v356 = iconst.i32 6
//     v357 = sshr v350, v356  ; v356 = 6
//     v358 = imul v355, v357
//     v359 = isub v347, v358
//     v360 = sshr v347, v356  ; v356 = 6
//     v361 = imul v355, v360
//     v362 = iadd v350, v361
//     v363 = iconst.i32 1023
//     v364 = imul v355, v363  ; v363 = 1023
//     v365 = isub v353, v364
//     v366 = icmp slt v365, v279  ; v279 = 0
//     v367 = select v366, v280, v281  ; v280 = -1, v281 = 1
//     v368 = iconst.i32 7
//     v369 = sshr v362, v368  ; v368 = 7
//     v370 = imul v367, v369
//     v371 = isub v359, v370
//     v372 = sshr v359, v368  ; v368 = 7
//     v373 = imul v367, v372
//     v374 = iadd v362, v373
//     v375 = iconst.i32 511
//     v376 = imul v367, v375  ; v375 = 511
//     v377 = isub v365, v376
//     v378 = icmp slt v377, v279  ; v279 = 0
//     v379 = select v378, v280, v281  ; v280 = -1, v281 = 1
//     v380 = iconst.i32 8
//     v381 = sshr v374, v380  ; v380 = 8
//     v382 = imul v379, v381
//     v383 = isub v371, v382
//     v384 = sshr v371, v380  ; v380 = 8
//     v385 = imul v379, v384
//     v386 = iadd v374, v385
//     v387 = iconst.i32 255
//     v388 = imul v379, v387  ; v387 = 255
//     v389 = isub v377, v388
//     v390 = icmp slt v389, v279  ; v279 = 0
//     v391 = select v390, v280, v281  ; v280 = -1, v281 = 1
//     v392 = iconst.i32 9
//     v393 = sshr v386, v392  ; v392 = 9
//     v394 = imul v391, v393
//     v395 = isub v383, v394
//     v396 = sshr v383, v392  ; v392 = 9
//     v397 = imul v391, v396
//     v398 = iadd v386, v397
//     v399 = iconst.i32 127
//     v400 = imul v391, v399  ; v399 = 127
//     v401 = isub v389, v400
//     v402 = icmp slt v401, v279  ; v279 = 0
//     v403 = select v402, v280, v281  ; v280 = -1, v281 = 1
//     v404 = iconst.i32 10
//     v405 = sshr v398, v404  ; v404 = 10
//     v406 = imul v403, v405
//     v407 = isub v395, v406
//     v408 = sshr v395, v404  ; v404 = 10
//     v409 = imul v403, v408
//     v410 = iadd v398, v409
//     v411 = iconst.i32 63
//     v412 = imul v403, v411  ; v411 = 63
//     v413 = isub v401, v412
//     v414 = icmp slt v413, v279  ; v279 = 0
//     v415 = select v414, v280, v281  ; v280 = -1, v281 = 1
//     v416 = iconst.i32 11
//     v417 = sshr v410, v416  ; v416 = 11
//     v418 = imul v415, v417
//     v419 = isub v407, v418
//     v420 = sshr v407, v416  ; v416 = 11
//     v421 = imul v415, v420
//     v422 = iadd v410, v421
//     v423 = iconst.i32 31
//     v424 = imul v415, v423  ; v423 = 31
//     v425 = isub v413, v424
//     v426 = icmp slt v425, v279  ; v279 = 0
//     v427 = select v426, v280, v281  ; v280 = -1, v281 = 1
//     v428 = iconst.i32 12
//     v429 = sshr v422, v428  ; v428 = 12
//     v430 = imul v427, v429
//     v431 = isub v419, v430
//     v432 = sshr v419, v428  ; v428 = 12
//     v433 = imul v427, v432
//     v434 = iadd v422, v433
//     v435 = iconst.i32 15
//     v436 = imul v427, v435  ; v435 = 15
//     v437 = isub v425, v436
//     v438 = icmp slt v437, v279  ; v279 = 0
//     v439 = select v438, v280, v281  ; v280 = -1, v281 = 1
//     v440 = iconst.i32 13
//     v441 = sshr v434, v440  ; v440 = 13
//     v442 = imul v439, v441
//     v443 = isub v431, v442
//     v444 = sshr v431, v440  ; v440 = 13
//     v445 = imul v439, v444
//     v446 = iadd v434, v445
//     v447 = iconst.i32 7
//     v448 = imul v439, v447  ; v447 = 7
//     v449 = isub v437, v448
//     v450 = icmp slt v449, v279  ; v279 = 0
//     v451 = select v450, v280, v281  ; v280 = -1, v281 = 1
//     v452 = iconst.i32 14
//     v453 = sshr v446, v452  ; v452 = 14
//     v454 = imul v451, v453
//     v455 = isub v443, v454
//     v456 = sshr v443, v452  ; v452 = 14
//     v457 = imul v451, v456
//     v458 = iadd v446, v457
//     v459 = iconst.i32 3
//     v460 = imul v451, v459  ; v459 = 3
//     v461 = isub v449, v460
//     v462 = icmp slt v461, v279  ; v279 = 0
//     v463 = select v462, v280, v281  ; v280 = -1, v281 = 1
//     v464 = iconst.i32 15
//     v465 = sshr v458, v464  ; v464 = 15
//     v466 = imul v463, v465
//     v467 = isub v455, v466
//     v468 = sshr v455, v464  ; v464 = 15
//     v469 = imul v463, v468
//     v470 = iadd v458, v469
//     v471 = iconst.i32 1
//     v472 = imul v463, v471  ; v471 = 1
//     v473 = isub v461, v472
//     v474 = iconst.i32 0x0001_a592
//     v475 = sextend.i64 v470
//     v476 = sextend.i64 v474  ; v474 = 0x0001_a592
//     v477 = imul v475, v476
//     v478 = iconst.i64 16
//     v479 = sshr v477, v478  ; v478 = 16
//     v480 = ireduce.i32 v479
//     v481 = sextend.i64 v467
//     v482 = imul v481, v476
//     v483 = sshr v482, v478  ; v478 = 16
//     v484 = ireduce.i32 v483
//     v485 = iconst.i32 0
//     v486 = iconst.i32 1
//     v487 = iconst.i32 2
//     v488 = iconst.i32 3
//     v489 = icmp eq v276, v485  ; v485 = 0
//     v490 = icmp eq v276, v486  ; v486 = 1
//     v491 = icmp eq v276, v487  ; v487 = 2
//     v492 = icmp eq v276, v488  ; v488 = 3
//     v493 = ineg v484
//     v494 = select v490, v493, v484
//     v495 = select v491, v493, v494
//     v496 = select v492, v484, v495
//     v497 = select v489, v484, v496
//     v498 = sextend.i64 v250
//     v499 = iconst.i64 16
//     v500 = ishl v498, v499  ; v499 = 16
//     v501 = sextend.i64 v497
//     v502 = sdiv v500, v501
//     v503 = ireduce.i32 v502
//     return v503
//
// block1:
//     v504 = iconst.i32 0
//     return v504  ; v504 = 0
// }
// run: ~= 1 
