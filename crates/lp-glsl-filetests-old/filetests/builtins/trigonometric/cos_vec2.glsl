// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    vec2 angles = vec2(0.0, 3.141592654); // 0, π
    return cos(angles);
}

// function u0:0(i32 sret) system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = %cosf sig0
//
// block0(v0: i32):
//     v7 = iconst.i32 0
//     v8 = iconst.i32 0x0003_243f
//     v9 = iconst.i32 0x0003_243f
//     v10 = iconst.i32 0x0001_921f
//     v11 = iconst.i32 0x0006_487e
//     v12 = iconst.i32 0
//     v13 = iadd v9, v10  ; v9 = 0x0003_243f, v10 = 0x0001_921f
//     v14 = iconst.i32 1
//     v15 = iconst.i32 2
//     v16 = iconst.i32 3
//     v17 = icmp sle v7, v10  ; v7 = 0, v10 = 0x0001_921f
//     v18 = icmp sle v7, v9  ; v7 = 0, v9 = 0x0003_243f
//     v19 = icmp sle v7, v13  ; v7 = 0
//     v20 = icmp sgt v7, v13  ; v7 = 0
//     v21 = isub v9, v7  ; v9 = 0x0003_243f, v7 = 0
//     v22 = isub v7, v9  ; v7 = 0, v9 = 0x0003_243f
//     v23 = isub v11, v7  ; v11 = 0x0006_487e, v7 = 0
//     v24 = select v18, v21, v7  ; v7 = 0
//     v25 = select v19, v22, v24
//     v26 = select v20, v23, v25
//     v27 = bnot v17
//     v28 = band v18, v27
//     v29 = select v17, v12, v12  ; v12 = 0, v12 = 0
//     v30 = select v28, v14, v29  ; v14 = 1
//     v31 = bnot v18
//     v32 = band v19, v31
//     v33 = select v32, v15, v30  ; v15 = 2
//     v34 = select v20, v16, v33  ; v16 = 3
//     v35 = iconst.i32 0x9b74
//     v36 = iconst.i32 0
//     v37 = iconst.i32 0
//     v38 = iconst.i32 -1
//     v39 = iconst.i32 1
//     v40 = icmp slt v26, v37  ; v37 = 0
//     v41 = select v40, v38, v39  ; v38 = -1, v39 = 1
//     v42 = iconst.i32 0
//     v43 = sshr v36, v42  ; v36 = 0, v42 = 0
//     v44 = imul v41, v43
//     v45 = isub v35, v44  ; v35 = 0x9b74
//     v46 = sshr v35, v42  ; v35 = 0x9b74, v42 = 0
//     v47 = imul v41, v46
//     v48 = iadd v36, v47  ; v36 = 0
//     v49 = iconst.i32 0xc90f
//     v50 = imul v41, v49  ; v49 = 0xc90f
//     v51 = isub v26, v50
//     v52 = icmp slt v51, v37  ; v37 = 0
//     v53 = select v52, v38, v39  ; v38 = -1, v39 = 1
//     v54 = iconst.i32 1
//     v55 = sshr v48, v54  ; v54 = 1
//     v56 = imul v53, v55
//     v57 = isub v45, v56
//     v58 = sshr v45, v54  ; v54 = 1
//     v59 = imul v53, v58
//     v60 = iadd v48, v59
//     v61 = iconst.i32 0x76b1
//     v62 = imul v53, v61  ; v61 = 0x76b1
//     v63 = isub v51, v62
//     v64 = icmp slt v63, v37  ; v37 = 0
//     v65 = select v64, v38, v39  ; v38 = -1, v39 = 1
//     v66 = iconst.i32 2
//     v67 = sshr v60, v66  ; v66 = 2
//     v68 = imul v65, v67
//     v69 = isub v57, v68
//     v70 = sshr v57, v66  ; v66 = 2
//     v71 = imul v65, v70
//     v72 = iadd v60, v71
//     v73 = iconst.i32 0x3eb6
//     v74 = imul v65, v73  ; v73 = 0x3eb6
//     v75 = isub v63, v74
//     v76 = icmp slt v75, v37  ; v37 = 0
//     v77 = select v76, v38, v39  ; v38 = -1, v39 = 1
//     v78 = iconst.i32 3
//     v79 = sshr v72, v78  ; v78 = 3
//     v80 = imul v77, v79
//     v81 = isub v69, v80
//     v82 = sshr v69, v78  ; v78 = 3
//     v83 = imul v77, v82
//     v84 = iadd v72, v83
//     v85 = iconst.i32 8149
//     v86 = imul v77, v85  ; v85 = 8149
//     v87 = isub v75, v86
//     v88 = icmp slt v87, v37  ; v37 = 0
//     v89 = select v88, v38, v39  ; v38 = -1, v39 = 1
//     v90 = iconst.i32 4
//     v91 = sshr v84, v90  ; v90 = 4
//     v92 = imul v89, v91
//     v93 = isub v81, v92
//     v94 = sshr v81, v90  ; v90 = 4
//     v95 = imul v89, v94
//     v96 = iadd v84, v95
//     v97 = iconst.i32 4090
//     v98 = imul v89, v97  ; v97 = 4090
//     v99 = isub v87, v98
//     v100 = icmp slt v99, v37  ; v37 = 0
//     v101 = select v100, v38, v39  ; v38 = -1, v39 = 1
//     v102 = iconst.i32 5
//     v103 = sshr v96, v102  ; v102 = 5
//     v104 = imul v101, v103
//     v105 = isub v93, v104
//     v106 = sshr v93, v102  ; v102 = 5
//     v107 = imul v101, v106
//     v108 = iadd v96, v107
//     v109 = iconst.i32 2047
//     v110 = imul v101, v109  ; v109 = 2047
//     v111 = isub v99, v110
//     v112 = icmp slt v111, v37  ; v37 = 0
//     v113 = select v112, v38, v39  ; v38 = -1, v39 = 1
//     v114 = iconst.i32 6
//     v115 = sshr v108, v114  ; v114 = 6
//     v116 = imul v113, v115
//     v117 = isub v105, v116
//     v118 = sshr v105, v114  ; v114 = 6
//     v119 = imul v113, v118
//     v120 = iadd v108, v119
//     v121 = iconst.i32 1023
//     v122 = imul v113, v121  ; v121 = 1023
//     v123 = isub v111, v122
//     v124 = icmp slt v123, v37  ; v37 = 0
//     v125 = select v124, v38, v39  ; v38 = -1, v39 = 1
//     v126 = iconst.i32 7
//     v127 = sshr v120, v126  ; v126 = 7
//     v128 = imul v125, v127
//     v129 = isub v117, v128
//     v130 = sshr v117, v126  ; v126 = 7
//     v131 = imul v125, v130
//     v132 = iadd v120, v131
//     v133 = iconst.i32 511
//     v134 = imul v125, v133  ; v133 = 511
//     v135 = isub v123, v134
//     v136 = icmp slt v135, v37  ; v37 = 0
//     v137 = select v136, v38, v39  ; v38 = -1, v39 = 1
//     v138 = iconst.i32 8
//     v139 = sshr v132, v138  ; v138 = 8
//     v140 = imul v137, v139
//     v141 = isub v129, v140
//     v142 = sshr v129, v138  ; v138 = 8
//     v143 = imul v137, v142
//     v144 = iadd v132, v143
//     v145 = iconst.i32 255
//     v146 = imul v137, v145  ; v145 = 255
//     v147 = isub v135, v146
//     v148 = icmp slt v147, v37  ; v37 = 0
//     v149 = select v148, v38, v39  ; v38 = -1, v39 = 1
//     v150 = iconst.i32 9
//     v151 = sshr v144, v150  ; v150 = 9
//     v152 = imul v149, v151
//     v153 = isub v141, v152
//     v154 = sshr v141, v150  ; v150 = 9
//     v155 = imul v149, v154
//     v156 = iadd v144, v155
//     v157 = iconst.i32 127
//     v158 = imul v149, v157  ; v157 = 127
//     v159 = isub v147, v158
//     v160 = icmp slt v159, v37  ; v37 = 0
//     v161 = select v160, v38, v39  ; v38 = -1, v39 = 1
//     v162 = iconst.i32 10
//     v163 = sshr v156, v162  ; v162 = 10
//     v164 = imul v161, v163
//     v165 = isub v153, v164
//     v166 = sshr v153, v162  ; v162 = 10
//     v167 = imul v161, v166
//     v168 = iadd v156, v167
//     v169 = iconst.i32 63
//     v170 = imul v161, v169  ; v169 = 63
//     v171 = isub v159, v170
//     v172 = icmp slt v171, v37  ; v37 = 0
//     v173 = select v172, v38, v39  ; v38 = -1, v39 = 1
//     v174 = iconst.i32 11
//     v175 = sshr v168, v174  ; v174 = 11
//     v176 = imul v173, v175
//     v177 = isub v165, v176
//     v178 = sshr v165, v174  ; v174 = 11
//     v179 = imul v173, v178
//     v180 = iadd v168, v179
//     v181 = iconst.i32 31
//     v182 = imul v173, v181  ; v181 = 31
//     v183 = isub v171, v182
//     v184 = icmp slt v183, v37  ; v37 = 0
//     v185 = select v184, v38, v39  ; v38 = -1, v39 = 1
//     v186 = iconst.i32 12
//     v187 = sshr v180, v186  ; v186 = 12
//     v188 = imul v185, v187
//     v189 = isub v177, v188
//     v190 = sshr v177, v186  ; v186 = 12
//     v191 = imul v185, v190
//     v192 = iadd v180, v191
//     v193 = iconst.i32 15
//     v194 = imul v185, v193  ; v193 = 15
//     v195 = isub v183, v194
//     v196 = icmp slt v195, v37  ; v37 = 0
//     v197 = select v196, v38, v39  ; v38 = -1, v39 = 1
//     v198 = iconst.i32 13
//     v199 = sshr v192, v198  ; v198 = 13
//     v200 = imul v197, v199
//     v201 = isub v189, v200
//     v202 = sshr v189, v198  ; v198 = 13
//     v203 = imul v197, v202
//     v204 = iadd v192, v203
//     v205 = iconst.i32 7
//     v206 = imul v197, v205  ; v205 = 7
//     v207 = isub v195, v206
//     v208 = icmp slt v207, v37  ; v37 = 0
//     v209 = select v208, v38, v39  ; v38 = -1, v39 = 1
//     v210 = iconst.i32 14
//     v211 = sshr v204, v210  ; v210 = 14
//     v212 = imul v209, v211
//     v213 = isub v201, v212
//     v214 = sshr v201, v210  ; v210 = 14
//     v215 = imul v209, v214
//     v216 = iadd v204, v215
//     v217 = iconst.i32 3
//     v218 = imul v209, v217  ; v217 = 3
//     v219 = isub v207, v218
//     v220 = icmp slt v219, v37  ; v37 = 0
//     v221 = select v220, v38, v39  ; v38 = -1, v39 = 1
//     v222 = iconst.i32 15
//     v223 = sshr v216, v222  ; v222 = 15
//     v224 = imul v221, v223
//     v225 = isub v213, v224
//     v226 = sshr v213, v222  ; v222 = 15
//     v227 = imul v221, v226
//     v228 = iadd v216, v227
//     v229 = iconst.i32 1
//     v230 = imul v221, v229  ; v229 = 1
//     v231 = isub v219, v230
//     v232 = iconst.i32 0x0001_a592
//     v233 = sextend.i64 v228
//     v234 = sextend.i64 v232  ; v232 = 0x0001_a592
//     v235 = imul v233, v234
//     v236 = iconst.i64 16
//     v237 = sshr v235, v236  ; v236 = 16
//     v238 = ireduce.i32 v237
//     v239 = sextend.i64 v225
//     v240 = imul v239, v234
//     v241 = sshr v240, v236  ; v236 = 16
//     v242 = ireduce.i32 v241
//     v243 = iconst.i32 0
//     v244 = iconst.i32 1
//     v245 = iconst.i32 2
//     v246 = iconst.i32 3
//     v247 = icmp eq v34, v243  ; v243 = 0
//     v248 = icmp eq v34, v244  ; v244 = 1
//     v249 = icmp eq v34, v245  ; v245 = 2
//     v250 = icmp eq v34, v246  ; v246 = 3
//     v251 = ineg v242
//     v252 = select v248, v251, v242
//     v253 = select v249, v251, v252
//     v254 = select v250, v242, v253
//     v255 = select v247, v242, v254
//     v256 = iconst.i32 0x0003_243f
//     v257 = iconst.i32 0x0001_921f
//     v258 = iconst.i32 0x0006_487e
//     v259 = iconst.i32 0
//     v260 = iadd v256, v257  ; v256 = 0x0003_243f, v257 = 0x0001_921f
//     v261 = iconst.i32 1
//     v262 = iconst.i32 2
//     v263 = iconst.i32 3
//     v264 = icmp sle v8, v257  ; v8 = 0x0003_243f, v257 = 0x0001_921f
//     v265 = icmp sle v8, v256  ; v8 = 0x0003_243f, v256 = 0x0003_243f
//     v266 = icmp sle v8, v260  ; v8 = 0x0003_243f
//     v267 = icmp sgt v8, v260  ; v8 = 0x0003_243f
//     v268 = isub v256, v8  ; v256 = 0x0003_243f, v8 = 0x0003_243f
//     v269 = isub v8, v256  ; v8 = 0x0003_243f, v256 = 0x0003_243f
//     v270 = isub v258, v8  ; v258 = 0x0006_487e, v8 = 0x0003_243f
//     v271 = select v265, v268, v8  ; v8 = 0x0003_243f
//     v272 = select v266, v269, v271
//     v273 = select v267, v270, v272
//     v274 = bnot v264
//     v275 = band v265, v274
//     v276 = select v264, v259, v259  ; v259 = 0, v259 = 0
//     v277 = select v275, v261, v276  ; v261 = 1
//     v278 = bnot v265
//     v279 = band v266, v278
//     v280 = select v279, v262, v277  ; v262 = 2
//     v281 = select v267, v263, v280  ; v263 = 3
//     v282 = iconst.i32 0x9b74
//     v283 = iconst.i32 0
//     v284 = iconst.i32 0
//     v285 = iconst.i32 -1
//     v286 = iconst.i32 1
//     v287 = icmp slt v273, v284  ; v284 = 0
//     v288 = select v287, v285, v286  ; v285 = -1, v286 = 1
//     v289 = iconst.i32 0
//     v290 = sshr v283, v289  ; v283 = 0, v289 = 0
//     v291 = imul v288, v290
//     v292 = isub v282, v291  ; v282 = 0x9b74
//     v293 = sshr v282, v289  ; v282 = 0x9b74, v289 = 0
//     v294 = imul v288, v293
//     v295 = iadd v283, v294  ; v283 = 0
//     v296 = iconst.i32 0xc90f
//     v297 = imul v288, v296  ; v296 = 0xc90f
//     v298 = isub v273, v297
//     v299 = icmp slt v298, v284  ; v284 = 0
//     v300 = select v299, v285, v286  ; v285 = -1, v286 = 1
//     v301 = iconst.i32 1
//     v302 = sshr v295, v301  ; v301 = 1
//     v303 = imul v300, v302
//     v304 = isub v292, v303
//     v305 = sshr v292, v301  ; v301 = 1
//     v306 = imul v300, v305
//     v307 = iadd v295, v306
//     v308 = iconst.i32 0x76b1
//     v309 = imul v300, v308  ; v308 = 0x76b1
//     v310 = isub v298, v309
//     v311 = icmp slt v310, v284  ; v284 = 0
//     v312 = select v311, v285, v286  ; v285 = -1, v286 = 1
//     v313 = iconst.i32 2
//     v314 = sshr v307, v313  ; v313 = 2
//     v315 = imul v312, v314
//     v316 = isub v304, v315
//     v317 = sshr v304, v313  ; v313 = 2
//     v318 = imul v312, v317
//     v319 = iadd v307, v318
//     v320 = iconst.i32 0x3eb6
//     v321 = imul v312, v320  ; v320 = 0x3eb6
//     v322 = isub v310, v321
//     v323 = icmp slt v322, v284  ; v284 = 0
//     v324 = select v323, v285, v286  ; v285 = -1, v286 = 1
//     v325 = iconst.i32 3
//     v326 = sshr v319, v325  ; v325 = 3
//     v327 = imul v324, v326
//     v328 = isub v316, v327
//     v329 = sshr v316, v325  ; v325 = 3
//     v330 = imul v324, v329
//     v331 = iadd v319, v330
//     v332 = iconst.i32 8149
//     v333 = imul v324, v332  ; v332 = 8149
//     v334 = isub v322, v333
//     v335 = icmp slt v334, v284  ; v284 = 0
//     v336 = select v335, v285, v286  ; v285 = -1, v286 = 1
//     v337 = iconst.i32 4
//     v338 = sshr v331, v337  ; v337 = 4
//     v339 = imul v336, v338
//     v340 = isub v328, v339
//     v341 = sshr v328, v337  ; v337 = 4
//     v342 = imul v336, v341
//     v343 = iadd v331, v342
//     v344 = iconst.i32 4090
//     v345 = imul v336, v344  ; v344 = 4090
//     v346 = isub v334, v345
//     v347 = icmp slt v346, v284  ; v284 = 0
//     v348 = select v347, v285, v286  ; v285 = -1, v286 = 1
//     v349 = iconst.i32 5
//     v350 = sshr v343, v349  ; v349 = 5
//     v351 = imul v348, v350
//     v352 = isub v340, v351
//     v353 = sshr v340, v349  ; v349 = 5
//     v354 = imul v348, v353
//     v355 = iadd v343, v354
//     v356 = iconst.i32 2047
//     v357 = imul v348, v356  ; v356 = 2047
//     v358 = isub v346, v357
//     v359 = icmp slt v358, v284  ; v284 = 0
//     v360 = select v359, v285, v286  ; v285 = -1, v286 = 1
//     v361 = iconst.i32 6
//     v362 = sshr v355, v361  ; v361 = 6
//     v363 = imul v360, v362
//     v364 = isub v352, v363
//     v365 = sshr v352, v361  ; v361 = 6
//     v366 = imul v360, v365
//     v367 = iadd v355, v366
//     v368 = iconst.i32 1023
//     v369 = imul v360, v368  ; v368 = 1023
//     v370 = isub v358, v369
//     v371 = icmp slt v370, v284  ; v284 = 0
//     v372 = select v371, v285, v286  ; v285 = -1, v286 = 1
//     v373 = iconst.i32 7
//     v374 = sshr v367, v373  ; v373 = 7
//     v375 = imul v372, v374
//     v376 = isub v364, v375
//     v377 = sshr v364, v373  ; v373 = 7
//     v378 = imul v372, v377
//     v379 = iadd v367, v378
//     v380 = iconst.i32 511
//     v381 = imul v372, v380  ; v380 = 511
//     v382 = isub v370, v381
//     v383 = icmp slt v382, v284  ; v284 = 0
//     v384 = select v383, v285, v286  ; v285 = -1, v286 = 1
//     v385 = iconst.i32 8
//     v386 = sshr v379, v385  ; v385 = 8
//     v387 = imul v384, v386
//     v388 = isub v376, v387
//     v389 = sshr v376, v385  ; v385 = 8
//     v390 = imul v384, v389
//     v391 = iadd v379, v390
//     v392 = iconst.i32 255
//     v393 = imul v384, v392  ; v392 = 255
//     v394 = isub v382, v393
//     v395 = icmp slt v394, v284  ; v284 = 0
//     v396 = select v395, v285, v286  ; v285 = -1, v286 = 1
//     v397 = iconst.i32 9
//     v398 = sshr v391, v397  ; v397 = 9
//     v399 = imul v396, v398
//     v400 = isub v388, v399
//     v401 = sshr v388, v397  ; v397 = 9
//     v402 = imul v396, v401
//     v403 = iadd v391, v402
//     v404 = iconst.i32 127
//     v405 = imul v396, v404  ; v404 = 127
//     v406 = isub v394, v405
//     v407 = icmp slt v406, v284  ; v284 = 0
//     v408 = select v407, v285, v286  ; v285 = -1, v286 = 1
//     v409 = iconst.i32 10
//     v410 = sshr v403, v409  ; v409 = 10
//     v411 = imul v408, v410
//     v412 = isub v400, v411
//     v413 = sshr v400, v409  ; v409 = 10
//     v414 = imul v408, v413
//     v415 = iadd v403, v414
//     v416 = iconst.i32 63
//     v417 = imul v408, v416  ; v416 = 63
//     v418 = isub v406, v417
//     v419 = icmp slt v418, v284  ; v284 = 0
//     v420 = select v419, v285, v286  ; v285 = -1, v286 = 1
//     v421 = iconst.i32 11
//     v422 = sshr v415, v421  ; v421 = 11
//     v423 = imul v420, v422
//     v424 = isub v412, v423
//     v425 = sshr v412, v421  ; v421 = 11
//     v426 = imul v420, v425
//     v427 = iadd v415, v426
//     v428 = iconst.i32 31
//     v429 = imul v420, v428  ; v428 = 31
//     v430 = isub v418, v429
//     v431 = icmp slt v430, v284  ; v284 = 0
//     v432 = select v431, v285, v286  ; v285 = -1, v286 = 1
//     v433 = iconst.i32 12
//     v434 = sshr v427, v433  ; v433 = 12
//     v435 = imul v432, v434
//     v436 = isub v424, v435
//     v437 = sshr v424, v433  ; v433 = 12
//     v438 = imul v432, v437
//     v439 = iadd v427, v438
//     v440 = iconst.i32 15
//     v441 = imul v432, v440  ; v440 = 15
//     v442 = isub v430, v441
//     v443 = icmp slt v442, v284  ; v284 = 0
//     v444 = select v443, v285, v286  ; v285 = -1, v286 = 1
//     v445 = iconst.i32 13
//     v446 = sshr v439, v445  ; v445 = 13
//     v447 = imul v444, v446
//     v448 = isub v436, v447
//     v449 = sshr v436, v445  ; v445 = 13
//     v450 = imul v444, v449
//     v451 = iadd v439, v450
//     v452 = iconst.i32 7
//     v453 = imul v444, v452  ; v452 = 7
//     v454 = isub v442, v453
//     v455 = icmp slt v454, v284  ; v284 = 0
//     v456 = select v455, v285, v286  ; v285 = -1, v286 = 1
//     v457 = iconst.i32 14
//     v458 = sshr v451, v457  ; v457 = 14
//     v459 = imul v456, v458
//     v460 = isub v448, v459
//     v461 = sshr v448, v457  ; v457 = 14
//     v462 = imul v456, v461
//     v463 = iadd v451, v462
//     v464 = iconst.i32 3
//     v465 = imul v456, v464  ; v464 = 3
//     v466 = isub v454, v465
//     v467 = icmp slt v466, v284  ; v284 = 0
//     v468 = select v467, v285, v286  ; v285 = -1, v286 = 1
//     v469 = iconst.i32 15
//     v470 = sshr v463, v469  ; v469 = 15
//     v471 = imul v468, v470
//     v472 = isub v460, v471
//     v473 = sshr v460, v469  ; v469 = 15
//     v474 = imul v468, v473
//     v475 = iadd v463, v474
//     v476 = iconst.i32 1
//     v477 = imul v468, v476  ; v476 = 1
//     v478 = isub v466, v477
//     v479 = iconst.i32 0x0001_a592
//     v480 = sextend.i64 v475
//     v481 = sextend.i64 v479  ; v479 = 0x0001_a592
//     v482 = imul v480, v481
//     v483 = iconst.i64 16
//     v484 = sshr v482, v483  ; v483 = 16
//     v485 = ireduce.i32 v484
//     v486 = sextend.i64 v472
//     v487 = imul v486, v481
//     v488 = sshr v487, v483  ; v483 = 16
//     v489 = ireduce.i32 v488
//     v490 = iconst.i32 0
//     v491 = iconst.i32 1
//     v492 = iconst.i32 2
//     v493 = iconst.i32 3
//     v494 = icmp eq v281, v490  ; v490 = 0
//     v495 = icmp eq v281, v491  ; v491 = 1
//     v496 = icmp eq v281, v492  ; v492 = 2
//     v497 = icmp eq v281, v493  ; v493 = 3
//     v498 = ineg v489
//     v499 = select v495, v498, v489
//     v500 = select v496, v498, v499
//     v501 = select v497, v489, v500
//     v502 = select v494, v489, v501
//     store notrap aligned v255, v0
//     store notrap aligned v502, v0+4
//     return
//
// block1:
//     v503 = iconst.i32 0
//     store notrap aligned v503, v0  ; v503 = 0
//     v504 = iconst.i32 0
//     store notrap aligned v504, v0+4  ; v504 = 0
//     return
// }
// run: ≈ vec2(1.0, -1.0) (tolerance: 0.001)
