// test compile
// test run
// target riscv32.fixed32

vec3 main() {
    vec3 angles = vec3(0.0, 0.785398163, 1.570796327); // 0, π/4, π/2
    return tan(angles);
}

// function u0:0(i32 sret) system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = %tanf sig0
//
// block0(v0: i32):
//     v10 = iconst.i32 0
//     v11 = iconst.i32 0xc910
//     v12 = iconst.i32 0x0001_9220
//     v13 = iconst.i32 0x0003_243f
//     v14 = iconst.i32 0x0001_921f
//     v15 = iconst.i32 0x0006_487e
//     v16 = iconst.i32 0
//     v17 = iadd v13, v14  ; v13 = 0x0003_243f, v14 = 0x0001_921f
//     v18 = iconst.i32 1
//     v19 = iconst.i32 2
//     v20 = iconst.i32 3
//     v21 = icmp sle v10, v14  ; v10 = 0, v14 = 0x0001_921f
//     v22 = icmp sle v10, v13  ; v10 = 0, v13 = 0x0003_243f
//     v23 = icmp sle v10, v17  ; v10 = 0
//     v24 = icmp sgt v10, v17  ; v10 = 0
//     v25 = isub v13, v10  ; v13 = 0x0003_243f, v10 = 0
//     v26 = isub v10, v13  ; v10 = 0, v13 = 0x0003_243f
//     v27 = isub v15, v10  ; v15 = 0x0006_487e, v10 = 0
//     v28 = select v22, v25, v10  ; v10 = 0
//     v29 = select v23, v26, v28
//     v30 = select v24, v27, v29
//     v31 = bnot v21
//     v32 = band v22, v31
//     v33 = select v21, v16, v16  ; v16 = 0, v16 = 0
//     v34 = select v32, v18, v33  ; v18 = 1
//     v35 = bnot v22
//     v36 = band v23, v35
//     v37 = select v36, v19, v34  ; v19 = 2
//     v38 = select v24, v20, v37  ; v20 = 3
//     v39 = iconst.i32 0x9b74
//     v40 = iconst.i32 0
//     v41 = iconst.i32 0
//     v42 = iconst.i32 -1
//     v43 = iconst.i32 1
//     v44 = icmp slt v30, v41  ; v41 = 0
//     v45 = select v44, v42, v43  ; v42 = -1, v43 = 1
//     v46 = iconst.i32 0
//     v47 = sshr v40, v46  ; v40 = 0, v46 = 0
//     v48 = imul v45, v47
//     v49 = isub v39, v48  ; v39 = 0x9b74
//     v50 = sshr v39, v46  ; v39 = 0x9b74, v46 = 0
//     v51 = imul v45, v50
//     v52 = iadd v40, v51  ; v40 = 0
//     v53 = iconst.i32 0xc90f
//     v54 = imul v45, v53  ; v53 = 0xc90f
//     v55 = isub v30, v54
//     v56 = icmp slt v55, v41  ; v41 = 0
//     v57 = select v56, v42, v43  ; v42 = -1, v43 = 1
//     v58 = iconst.i32 1
//     v59 = sshr v52, v58  ; v58 = 1
//     v60 = imul v57, v59
//     v61 = isub v49, v60
//     v62 = sshr v49, v58  ; v58 = 1
//     v63 = imul v57, v62
//     v64 = iadd v52, v63
//     v65 = iconst.i32 0x76b1
//     v66 = imul v57, v65  ; v65 = 0x76b1
//     v67 = isub v55, v66
//     v68 = icmp slt v67, v41  ; v41 = 0
//     v69 = select v68, v42, v43  ; v42 = -1, v43 = 1
//     v70 = iconst.i32 2
//     v71 = sshr v64, v70  ; v70 = 2
//     v72 = imul v69, v71
//     v73 = isub v61, v72
//     v74 = sshr v61, v70  ; v70 = 2
//     v75 = imul v69, v74
//     v76 = iadd v64, v75
//     v77 = iconst.i32 0x3eb6
//     v78 = imul v69, v77  ; v77 = 0x3eb6
//     v79 = isub v67, v78
//     v80 = icmp slt v79, v41  ; v41 = 0
//     v81 = select v80, v42, v43  ; v42 = -1, v43 = 1
//     v82 = iconst.i32 3
//     v83 = sshr v76, v82  ; v82 = 3
//     v84 = imul v81, v83
//     v85 = isub v73, v84
//     v86 = sshr v73, v82  ; v82 = 3
//     v87 = imul v81, v86
//     v88 = iadd v76, v87
//     v89 = iconst.i32 8149
//     v90 = imul v81, v89  ; v89 = 8149
//     v91 = isub v79, v90
//     v92 = icmp slt v91, v41  ; v41 = 0
//     v93 = select v92, v42, v43  ; v42 = -1, v43 = 1
//     v94 = iconst.i32 4
//     v95 = sshr v88, v94  ; v94 = 4
//     v96 = imul v93, v95
//     v97 = isub v85, v96
//     v98 = sshr v85, v94  ; v94 = 4
//     v99 = imul v93, v98
//     v100 = iadd v88, v99
//     v101 = iconst.i32 4090
//     v102 = imul v93, v101  ; v101 = 4090
//     v103 = isub v91, v102
//     v104 = icmp slt v103, v41  ; v41 = 0
//     v105 = select v104, v42, v43  ; v42 = -1, v43 = 1
//     v106 = iconst.i32 5
//     v107 = sshr v100, v106  ; v106 = 5
//     v108 = imul v105, v107
//     v109 = isub v97, v108
//     v110 = sshr v97, v106  ; v106 = 5
//     v111 = imul v105, v110
//     v112 = iadd v100, v111
//     v113 = iconst.i32 2047
//     v114 = imul v105, v113  ; v113 = 2047
//     v115 = isub v103, v114
//     v116 = icmp slt v115, v41  ; v41 = 0
//     v117 = select v116, v42, v43  ; v42 = -1, v43 = 1
//     v118 = iconst.i32 6
//     v119 = sshr v112, v118  ; v118 = 6
//     v120 = imul v117, v119
//     v121 = isub v109, v120
//     v122 = sshr v109, v118  ; v118 = 6
//     v123 = imul v117, v122
//     v124 = iadd v112, v123
//     v125 = iconst.i32 1023
//     v126 = imul v117, v125  ; v125 = 1023
//     v127 = isub v115, v126
//     v128 = icmp slt v127, v41  ; v41 = 0
//     v129 = select v128, v42, v43  ; v42 = -1, v43 = 1
//     v130 = iconst.i32 7
//     v131 = sshr v124, v130  ; v130 = 7
//     v132 = imul v129, v131
//     v133 = isub v121, v132
//     v134 = sshr v121, v130  ; v130 = 7
//     v135 = imul v129, v134
//     v136 = iadd v124, v135
//     v137 = iconst.i32 511
//     v138 = imul v129, v137  ; v137 = 511
//     v139 = isub v127, v138
//     v140 = icmp slt v139, v41  ; v41 = 0
//     v141 = select v140, v42, v43  ; v42 = -1, v43 = 1
//     v142 = iconst.i32 8
//     v143 = sshr v136, v142  ; v142 = 8
//     v144 = imul v141, v143
//     v145 = isub v133, v144
//     v146 = sshr v133, v142  ; v142 = 8
//     v147 = imul v141, v146
//     v148 = iadd v136, v147
//     v149 = iconst.i32 255
//     v150 = imul v141, v149  ; v149 = 255
//     v151 = isub v139, v150
//     v152 = icmp slt v151, v41  ; v41 = 0
//     v153 = select v152, v42, v43  ; v42 = -1, v43 = 1
//     v154 = iconst.i32 9
//     v155 = sshr v148, v154  ; v154 = 9
//     v156 = imul v153, v155
//     v157 = isub v145, v156
//     v158 = sshr v145, v154  ; v154 = 9
//     v159 = imul v153, v158
//     v160 = iadd v148, v159
//     v161 = iconst.i32 127
//     v162 = imul v153, v161  ; v161 = 127
//     v163 = isub v151, v162
//     v164 = icmp slt v163, v41  ; v41 = 0
//     v165 = select v164, v42, v43  ; v42 = -1, v43 = 1
//     v166 = iconst.i32 10
//     v167 = sshr v160, v166  ; v166 = 10
//     v168 = imul v165, v167
//     v169 = isub v157, v168
//     v170 = sshr v157, v166  ; v166 = 10
//     v171 = imul v165, v170
//     v172 = iadd v160, v171
//     v173 = iconst.i32 63
//     v174 = imul v165, v173  ; v173 = 63
//     v175 = isub v163, v174
//     v176 = icmp slt v175, v41  ; v41 = 0
//     v177 = select v176, v42, v43  ; v42 = -1, v43 = 1
//     v178 = iconst.i32 11
//     v179 = sshr v172, v178  ; v178 = 11
//     v180 = imul v177, v179
//     v181 = isub v169, v180
//     v182 = sshr v169, v178  ; v178 = 11
//     v183 = imul v177, v182
//     v184 = iadd v172, v183
//     v185 = iconst.i32 31
//     v186 = imul v177, v185  ; v185 = 31
//     v187 = isub v175, v186
//     v188 = icmp slt v187, v41  ; v41 = 0
//     v189 = select v188, v42, v43  ; v42 = -1, v43 = 1
//     v190 = iconst.i32 12
//     v191 = sshr v184, v190  ; v190 = 12
//     v192 = imul v189, v191
//     v193 = isub v181, v192
//     v194 = sshr v181, v190  ; v190 = 12
//     v195 = imul v189, v194
//     v196 = iadd v184, v195
//     v197 = iconst.i32 15
//     v198 = imul v189, v197  ; v197 = 15
//     v199 = isub v187, v198
//     v200 = icmp slt v199, v41  ; v41 = 0
//     v201 = select v200, v42, v43  ; v42 = -1, v43 = 1
//     v202 = iconst.i32 13
//     v203 = sshr v196, v202  ; v202 = 13
//     v204 = imul v201, v203
//     v205 = isub v193, v204
//     v206 = sshr v193, v202  ; v202 = 13
//     v207 = imul v201, v206
//     v208 = iadd v196, v207
//     v209 = iconst.i32 7
//     v210 = imul v201, v209  ; v209 = 7
//     v211 = isub v199, v210
//     v212 = icmp slt v211, v41  ; v41 = 0
//     v213 = select v212, v42, v43  ; v42 = -1, v43 = 1
//     v214 = iconst.i32 14
//     v215 = sshr v208, v214  ; v214 = 14
//     v216 = imul v213, v215
//     v217 = isub v205, v216
//     v218 = sshr v205, v214  ; v214 = 14
//     v219 = imul v213, v218
//     v220 = iadd v208, v219
//     v221 = iconst.i32 3
//     v222 = imul v213, v221  ; v221 = 3
//     v223 = isub v211, v222
//     v224 = icmp slt v223, v41  ; v41 = 0
//     v225 = select v224, v42, v43  ; v42 = -1, v43 = 1
//     v226 = iconst.i32 15
//     v227 = sshr v220, v226  ; v226 = 15
//     v228 = imul v225, v227
//     v229 = isub v217, v228
//     v230 = sshr v217, v226  ; v226 = 15
//     v231 = imul v225, v230
//     v232 = iadd v220, v231
//     v233 = iconst.i32 1
//     v234 = imul v225, v233  ; v233 = 1
//     v235 = isub v223, v234
//     v236 = iconst.i32 0x0001_a592
//     v237 = sextend.i64 v232
//     v238 = sextend.i64 v236  ; v236 = 0x0001_a592
//     v239 = imul v237, v238
//     v240 = iconst.i64 16
//     v241 = sshr v239, v240  ; v240 = 16
//     v242 = ireduce.i32 v241
//     v243 = sextend.i64 v229
//     v244 = imul v243, v238
//     v245 = sshr v244, v240  ; v240 = 16
//     v246 = ireduce.i32 v245
//     v247 = iconst.i32 0
//     v248 = iconst.i32 1
//     v249 = iconst.i32 2
//     v250 = iconst.i32 3
//     v251 = icmp eq v38, v247  ; v247 = 0
//     v252 = icmp eq v38, v248  ; v248 = 1
//     v253 = icmp eq v38, v249  ; v249 = 2
//     v254 = icmp eq v38, v250  ; v250 = 3
//     v255 = ineg v242
//     v256 = select v253, v255, v242
//     v257 = select v254, v255, v256
//     v258 = select v252, v242, v257
//     v259 = select v251, v242, v258
//     v260 = iconst.i32 0x0003_243f
//     v261 = iconst.i32 0x0001_921f
//     v262 = iconst.i32 0x0006_487e
//     v263 = iconst.i32 0
//     v264 = iadd v260, v261  ; v260 = 0x0003_243f, v261 = 0x0001_921f
//     v265 = iconst.i32 1
//     v266 = iconst.i32 2
//     v267 = iconst.i32 3
//     v268 = icmp sle v10, v261  ; v10 = 0, v261 = 0x0001_921f
//     v269 = icmp sle v10, v260  ; v10 = 0, v260 = 0x0003_243f
//     v270 = icmp sle v10, v264  ; v10 = 0
//     v271 = icmp sgt v10, v264  ; v10 = 0
//     v272 = isub v260, v10  ; v260 = 0x0003_243f, v10 = 0
//     v273 = isub v10, v260  ; v10 = 0, v260 = 0x0003_243f
//     v274 = isub v262, v10  ; v262 = 0x0006_487e, v10 = 0
//     v275 = select v269, v272, v10  ; v10 = 0
//     v276 = select v270, v273, v275
//     v277 = select v271, v274, v276
//     v278 = bnot v268
//     v279 = band v269, v278
//     v280 = select v268, v263, v263  ; v263 = 0, v263 = 0
//     v281 = select v279, v265, v280  ; v265 = 1
//     v282 = bnot v269
//     v283 = band v270, v282
//     v284 = select v283, v266, v281  ; v266 = 2
//     v285 = select v271, v267, v284  ; v267 = 3
//     v286 = iconst.i32 0x9b74
//     v287 = iconst.i32 0
//     v288 = iconst.i32 0
//     v289 = iconst.i32 -1
//     v290 = iconst.i32 1
//     v291 = icmp slt v277, v288  ; v288 = 0
//     v292 = select v291, v289, v290  ; v289 = -1, v290 = 1
//     v293 = iconst.i32 0
//     v294 = sshr v287, v293  ; v287 = 0, v293 = 0
//     v295 = imul v292, v294
//     v296 = isub v286, v295  ; v286 = 0x9b74
//     v297 = sshr v286, v293  ; v286 = 0x9b74, v293 = 0
//     v298 = imul v292, v297
//     v299 = iadd v287, v298  ; v287 = 0
//     v300 = iconst.i32 0xc90f
//     v301 = imul v292, v300  ; v300 = 0xc90f
//     v302 = isub v277, v301
//     v303 = icmp slt v302, v288  ; v288 = 0
//     v304 = select v303, v289, v290  ; v289 = -1, v290 = 1
//     v305 = iconst.i32 1
//     v306 = sshr v299, v305  ; v305 = 1
//     v307 = imul v304, v306
//     v308 = isub v296, v307
//     v309 = sshr v296, v305  ; v305 = 1
//     v310 = imul v304, v309
//     v311 = iadd v299, v310
//     v312 = iconst.i32 0x76b1
//     v313 = imul v304, v312  ; v312 = 0x76b1
//     v314 = isub v302, v313
//     v315 = icmp slt v314, v288  ; v288 = 0
//     v316 = select v315, v289, v290  ; v289 = -1, v290 = 1
//     v317 = iconst.i32 2
//     v318 = sshr v311, v317  ; v317 = 2
//     v319 = imul v316, v318
//     v320 = isub v308, v319
//     v321 = sshr v308, v317  ; v317 = 2
//     v322 = imul v316, v321
//     v323 = iadd v311, v322
//     v324 = iconst.i32 0x3eb6
//     v325 = imul v316, v324  ; v324 = 0x3eb6
//     v326 = isub v314, v325
//     v327 = icmp slt v326, v288  ; v288 = 0
//     v328 = select v327, v289, v290  ; v289 = -1, v290 = 1
//     v329 = iconst.i32 3
//     v330 = sshr v323, v329  ; v329 = 3
//     v331 = imul v328, v330
//     v332 = isub v320, v331
//     v333 = sshr v320, v329  ; v329 = 3
//     v334 = imul v328, v333
//     v335 = iadd v323, v334
//     v336 = iconst.i32 8149
//     v337 = imul v328, v336  ; v336 = 8149
//     v338 = isub v326, v337
//     v339 = icmp slt v338, v288  ; v288 = 0
//     v340 = select v339, v289, v290  ; v289 = -1, v290 = 1
//     v341 = iconst.i32 4
//     v342 = sshr v335, v341  ; v341 = 4
//     v343 = imul v340, v342
//     v344 = isub v332, v343
//     v345 = sshr v332, v341  ; v341 = 4
//     v346 = imul v340, v345
//     v347 = iadd v335, v346
//     v348 = iconst.i32 4090
//     v349 = imul v340, v348  ; v348 = 4090
//     v350 = isub v338, v349
//     v351 = icmp slt v350, v288  ; v288 = 0
//     v352 = select v351, v289, v290  ; v289 = -1, v290 = 1
//     v353 = iconst.i32 5
//     v354 = sshr v347, v353  ; v353 = 5
//     v355 = imul v352, v354
//     v356 = isub v344, v355
//     v357 = sshr v344, v353  ; v353 = 5
//     v358 = imul v352, v357
//     v359 = iadd v347, v358
//     v360 = iconst.i32 2047
//     v361 = imul v352, v360  ; v360 = 2047
//     v362 = isub v350, v361
//     v363 = icmp slt v362, v288  ; v288 = 0
//     v364 = select v363, v289, v290  ; v289 = -1, v290 = 1
//     v365 = iconst.i32 6
//     v366 = sshr v359, v365  ; v365 = 6
//     v367 = imul v364, v366
//     v368 = isub v356, v367
//     v369 = sshr v356, v365  ; v365 = 6
//     v370 = imul v364, v369
//     v371 = iadd v359, v370
//     v372 = iconst.i32 1023
//     v373 = imul v364, v372  ; v372 = 1023
//     v374 = isub v362, v373
//     v375 = icmp slt v374, v288  ; v288 = 0
//     v376 = select v375, v289, v290  ; v289 = -1, v290 = 1
//     v377 = iconst.i32 7
//     v378 = sshr v371, v377  ; v377 = 7
//     v379 = imul v376, v378
//     v380 = isub v368, v379
//     v381 = sshr v368, v377  ; v377 = 7
//     v382 = imul v376, v381
//     v383 = iadd v371, v382
//     v384 = iconst.i32 511
//     v385 = imul v376, v384  ; v384 = 511
//     v386 = isub v374, v385
//     v387 = icmp slt v386, v288  ; v288 = 0
//     v388 = select v387, v289, v290  ; v289 = -1, v290 = 1
//     v389 = iconst.i32 8
//     v390 = sshr v383, v389  ; v389 = 8
//     v391 = imul v388, v390
//     v392 = isub v380, v391
//     v393 = sshr v380, v389  ; v389 = 8
//     v394 = imul v388, v393
//     v395 = iadd v383, v394
//     v396 = iconst.i32 255
//     v397 = imul v388, v396  ; v396 = 255
//     v398 = isub v386, v397
//     v399 = icmp slt v398, v288  ; v288 = 0
//     v400 = select v399, v289, v290  ; v289 = -1, v290 = 1
//     v401 = iconst.i32 9
//     v402 = sshr v395, v401  ; v401 = 9
//     v403 = imul v400, v402
//     v404 = isub v392, v403
//     v405 = sshr v392, v401  ; v401 = 9
//     v406 = imul v400, v405
//     v407 = iadd v395, v406
//     v408 = iconst.i32 127
//     v409 = imul v400, v408  ; v408 = 127
//     v410 = isub v398, v409
//     v411 = icmp slt v410, v288  ; v288 = 0
//     v412 = select v411, v289, v290  ; v289 = -1, v290 = 1
//     v413 = iconst.i32 10
//     v414 = sshr v407, v413  ; v413 = 10
//     v415 = imul v412, v414
//     v416 = isub v404, v415
//     v417 = sshr v404, v413  ; v413 = 10
//     v418 = imul v412, v417
//     v419 = iadd v407, v418
//     v420 = iconst.i32 63
//     v421 = imul v412, v420  ; v420 = 63
//     v422 = isub v410, v421
//     v423 = icmp slt v422, v288  ; v288 = 0
//     v424 = select v423, v289, v290  ; v289 = -1, v290 = 1
//     v425 = iconst.i32 11
//     v426 = sshr v419, v425  ; v425 = 11
//     v427 = imul v424, v426
//     v428 = isub v416, v427
//     v429 = sshr v416, v425  ; v425 = 11
//     v430 = imul v424, v429
//     v431 = iadd v419, v430
//     v432 = iconst.i32 31
//     v433 = imul v424, v432  ; v432 = 31
//     v434 = isub v422, v433
//     v435 = icmp slt v434, v288  ; v288 = 0
//     v436 = select v435, v289, v290  ; v289 = -1, v290 = 1
//     v437 = iconst.i32 12
//     v438 = sshr v431, v437  ; v437 = 12
//     v439 = imul v436, v438
//     v440 = isub v428, v439
//     v441 = sshr v428, v437  ; v437 = 12
//     v442 = imul v436, v441
//     v443 = iadd v431, v442
//     v444 = iconst.i32 15
//     v445 = imul v436, v444  ; v444 = 15
//     v446 = isub v434, v445
//     v447 = icmp slt v446, v288  ; v288 = 0
//     v448 = select v447, v289, v290  ; v289 = -1, v290 = 1
//     v449 = iconst.i32 13
//     v450 = sshr v443, v449  ; v449 = 13
//     v451 = imul v448, v450
//     v452 = isub v440, v451
//     v453 = sshr v440, v449  ; v449 = 13
//     v454 = imul v448, v453
//     v455 = iadd v443, v454
//     v456 = iconst.i32 7
//     v457 = imul v448, v456  ; v456 = 7
//     v458 = isub v446, v457
//     v459 = icmp slt v458, v288  ; v288 = 0
//     v460 = select v459, v289, v290  ; v289 = -1, v290 = 1
//     v461 = iconst.i32 14
//     v462 = sshr v455, v461  ; v461 = 14
//     v463 = imul v460, v462
//     v464 = isub v452, v463
//     v465 = sshr v452, v461  ; v461 = 14
//     v466 = imul v460, v465
//     v467 = iadd v455, v466
//     v468 = iconst.i32 3
//     v469 = imul v460, v468  ; v468 = 3
//     v470 = isub v458, v469
//     v471 = icmp slt v470, v288  ; v288 = 0
//     v472 = select v471, v289, v290  ; v289 = -1, v290 = 1
//     v473 = iconst.i32 15
//     v474 = sshr v467, v473  ; v473 = 15
//     v475 = imul v472, v474
//     v476 = isub v464, v475
//     v477 = sshr v464, v473  ; v473 = 15
//     v478 = imul v472, v477
//     v479 = iadd v467, v478
//     v480 = iconst.i32 1
//     v481 = imul v472, v480  ; v480 = 1
//     v482 = isub v470, v481
//     v483 = iconst.i32 0x0001_a592
//     v484 = sextend.i64 v479
//     v485 = sextend.i64 v483  ; v483 = 0x0001_a592
//     v486 = imul v484, v485
//     v487 = iconst.i64 16
//     v488 = sshr v486, v487  ; v487 = 16
//     v489 = ireduce.i32 v488
//     v490 = sextend.i64 v476
//     v491 = imul v490, v485
//     v492 = sshr v491, v487  ; v487 = 16
//     v493 = ireduce.i32 v492
//     v494 = iconst.i32 0
//     v495 = iconst.i32 1
//     v496 = iconst.i32 2
//     v497 = iconst.i32 3
//     v498 = icmp eq v285, v494  ; v494 = 0
//     v499 = icmp eq v285, v495  ; v495 = 1
//     v500 = icmp eq v285, v496  ; v496 = 2
//     v501 = icmp eq v285, v497  ; v497 = 3
//     v502 = ineg v493
//     v503 = select v499, v502, v493
//     v504 = select v500, v502, v503
//     v505 = select v501, v493, v504
//     v506 = select v498, v493, v505
//     v507 = sextend.i64 v259
//     v508 = iconst.i64 16
//     v509 = ishl v507, v508  ; v508 = 16
//     v510 = sextend.i64 v506
//     v511 = sdiv v509, v510
//     v512 = ireduce.i32 v511
//     v513 = iconst.i32 0x0003_243f
//     v514 = iconst.i32 0x0001_921f
//     v515 = iconst.i32 0x0006_487e
//     v516 = iconst.i32 0
//     v517 = iadd v513, v514  ; v513 = 0x0003_243f, v514 = 0x0001_921f
//     v518 = iconst.i32 1
//     v519 = iconst.i32 2
//     v520 = iconst.i32 3
//     v521 = icmp sle v11, v514  ; v11 = 0xc910, v514 = 0x0001_921f
//     v522 = icmp sle v11, v513  ; v11 = 0xc910, v513 = 0x0003_243f
//     v523 = icmp sle v11, v517  ; v11 = 0xc910
//     v524 = icmp sgt v11, v517  ; v11 = 0xc910
//     v525 = isub v513, v11  ; v513 = 0x0003_243f, v11 = 0xc910
//     v526 = isub v11, v513  ; v11 = 0xc910, v513 = 0x0003_243f
//     v527 = isub v515, v11  ; v515 = 0x0006_487e, v11 = 0xc910
//     v528 = select v522, v525, v11  ; v11 = 0xc910
//     v529 = select v523, v526, v528
//     v530 = select v524, v527, v529
//     v531 = bnot v521
//     v532 = band v522, v531
//     v533 = select v521, v516, v516  ; v516 = 0, v516 = 0
//     v534 = select v532, v518, v533  ; v518 = 1
//     v535 = bnot v522
//     v536 = band v523, v535
//     v537 = select v536, v519, v534  ; v519 = 2
//     v538 = select v524, v520, v537  ; v520 = 3
//     v539 = iconst.i32 0x9b74
//     v540 = iconst.i32 0
//     v541 = iconst.i32 0
//     v542 = iconst.i32 -1
//     v543 = iconst.i32 1
//     v544 = icmp slt v530, v541  ; v541 = 0
//     v545 = select v544, v542, v543  ; v542 = -1, v543 = 1
//     v546 = iconst.i32 0
//     v547 = sshr v540, v546  ; v540 = 0, v546 = 0
//     v548 = imul v545, v547
//     v549 = isub v539, v548  ; v539 = 0x9b74
//     v550 = sshr v539, v546  ; v539 = 0x9b74, v546 = 0
//     v551 = imul v545, v550
//     v552 = iadd v540, v551  ; v540 = 0
//     v553 = iconst.i32 0xc90f
//     v554 = imul v545, v553  ; v553 = 0xc90f
//     v555 = isub v530, v554
//     v556 = icmp slt v555, v541  ; v541 = 0
//     v557 = select v556, v542, v543  ; v542 = -1, v543 = 1
//     v558 = iconst.i32 1
//     v559 = sshr v552, v558  ; v558 = 1
//     v560 = imul v557, v559
//     v561 = isub v549, v560
//     v562 = sshr v549, v558  ; v558 = 1
//     v563 = imul v557, v562
//     v564 = iadd v552, v563
//     v565 = iconst.i32 0x76b1
//     v566 = imul v557, v565  ; v565 = 0x76b1
//     v567 = isub v555, v566
//     v568 = icmp slt v567, v541  ; v541 = 0
//     v569 = select v568, v542, v543  ; v542 = -1, v543 = 1
//     v570 = iconst.i32 2
//     v571 = sshr v564, v570  ; v570 = 2
//     v572 = imul v569, v571
//     v573 = isub v561, v572
//     v574 = sshr v561, v570  ; v570 = 2
//     v575 = imul v569, v574
//     v576 = iadd v564, v575
//     v577 = iconst.i32 0x3eb6
//     v578 = imul v569, v577  ; v577 = 0x3eb6
//     v579 = isub v567, v578
//     v580 = icmp slt v579, v541  ; v541 = 0
//     v581 = select v580, v542, v543  ; v542 = -1, v543 = 1
//     v582 = iconst.i32 3
//     v583 = sshr v576, v582  ; v582 = 3
//     v584 = imul v581, v583
//     v585 = isub v573, v584
//     v586 = sshr v573, v582  ; v582 = 3
//     v587 = imul v581, v586
//     v588 = iadd v576, v587
//     v589 = iconst.i32 8149
//     v590 = imul v581, v589  ; v589 = 8149
//     v591 = isub v579, v590
//     v592 = icmp slt v591, v541  ; v541 = 0
//     v593 = select v592, v542, v543  ; v542 = -1, v543 = 1
//     v594 = iconst.i32 4
//     v595 = sshr v588, v594  ; v594 = 4
//     v596 = imul v593, v595
//     v597 = isub v585, v596
//     v598 = sshr v585, v594  ; v594 = 4
//     v599 = imul v593, v598
//     v600 = iadd v588, v599
//     v601 = iconst.i32 4090
//     v602 = imul v593, v601  ; v601 = 4090
//     v603 = isub v591, v602
//     v604 = icmp slt v603, v541  ; v541 = 0
//     v605 = select v604, v542, v543  ; v542 = -1, v543 = 1
//     v606 = iconst.i32 5
//     v607 = sshr v600, v606  ; v606 = 5
//     v608 = imul v605, v607
//     v609 = isub v597, v608
//     v610 = sshr v597, v606  ; v606 = 5
//     v611 = imul v605, v610
//     v612 = iadd v600, v611
//     v613 = iconst.i32 2047
//     v614 = imul v605, v613  ; v613 = 2047
//     v615 = isub v603, v614
//     v616 = icmp slt v615, v541  ; v541 = 0
//     v617 = select v616, v542, v543  ; v542 = -1, v543 = 1
//     v618 = iconst.i32 6
//     v619 = sshr v612, v618  ; v618 = 6
//     v620 = imul v617, v619
//     v621 = isub v609, v620
//     v622 = sshr v609, v618  ; v618 = 6
//     v623 = imul v617, v622
//     v624 = iadd v612, v623
//     v625 = iconst.i32 1023
//     v626 = imul v617, v625  ; v625 = 1023
//     v627 = isub v615, v626
//     v628 = icmp slt v627, v541  ; v541 = 0
//     v629 = select v628, v542, v543  ; v542 = -1, v543 = 1
//     v630 = iconst.i32 7
//     v631 = sshr v624, v630  ; v630 = 7
//     v632 = imul v629, v631
//     v633 = isub v621, v632
//     v634 = sshr v621, v630  ; v630 = 7
//     v635 = imul v629, v634
//     v636 = iadd v624, v635
//     v637 = iconst.i32 511
//     v638 = imul v629, v637  ; v637 = 511
//     v639 = isub v627, v638
//     v640 = icmp slt v639, v541  ; v541 = 0
//     v641 = select v640, v542, v543  ; v542 = -1, v543 = 1
//     v642 = iconst.i32 8
//     v643 = sshr v636, v642  ; v642 = 8
//     v644 = imul v641, v643
//     v645 = isub v633, v644
//     v646 = sshr v633, v642  ; v642 = 8
//     v647 = imul v641, v646
//     v648 = iadd v636, v647
//     v649 = iconst.i32 255
//     v650 = imul v641, v649  ; v649 = 255
//     v651 = isub v639, v650
//     v652 = icmp slt v651, v541  ; v541 = 0
//     v653 = select v652, v542, v543  ; v542 = -1, v543 = 1
//     v654 = iconst.i32 9
//     v655 = sshr v648, v654  ; v654 = 9
//     v656 = imul v653, v655
//     v657 = isub v645, v656
//     v658 = sshr v645, v654  ; v654 = 9
//     v659 = imul v653, v658
//     v660 = iadd v648, v659
//     v661 = iconst.i32 127
//     v662 = imul v653, v661  ; v661 = 127
//     v663 = isub v651, v662
//     v664 = icmp slt v663, v541  ; v541 = 0
//     v665 = select v664, v542, v543  ; v542 = -1, v543 = 1
//     v666 = iconst.i32 10
//     v667 = sshr v660, v666  ; v666 = 10
//     v668 = imul v665, v667
//     v669 = isub v657, v668
//     v670 = sshr v657, v666  ; v666 = 10
//     v671 = imul v665, v670
//     v672 = iadd v660, v671
//     v673 = iconst.i32 63
//     v674 = imul v665, v673  ; v673 = 63
//     v675 = isub v663, v674
//     v676 = icmp slt v675, v541  ; v541 = 0
//     v677 = select v676, v542, v543  ; v542 = -1, v543 = 1
//     v678 = iconst.i32 11
//     v679 = sshr v672, v678  ; v678 = 11
//     v680 = imul v677, v679
//     v681 = isub v669, v680
//     v682 = sshr v669, v678  ; v678 = 11
//     v683 = imul v677, v682
//     v684 = iadd v672, v683
//     v685 = iconst.i32 31
//     v686 = imul v677, v685  ; v685 = 31
//     v687 = isub v675, v686
//     v688 = icmp slt v687, v541  ; v541 = 0
//     v689 = select v688, v542, v543  ; v542 = -1, v543 = 1
//     v690 = iconst.i32 12
//     v691 = sshr v684, v690  ; v690 = 12
//     v692 = imul v689, v691
//     v693 = isub v681, v692
//     v694 = sshr v681, v690  ; v690 = 12
//     v695 = imul v689, v694
//     v696 = iadd v684, v695
//     v697 = iconst.i32 15
//     v698 = imul v689, v697  ; v697 = 15
//     v699 = isub v687, v698
//     v700 = icmp slt v699, v541  ; v541 = 0
//     v701 = select v700, v542, v543  ; v542 = -1, v543 = 1
//     v702 = iconst.i32 13
//     v703 = sshr v696, v702  ; v702 = 13
//     v704 = imul v701, v703
//     v705 = isub v693, v704
//     v706 = sshr v693, v702  ; v702 = 13
//     v707 = imul v701, v706
//     v708 = iadd v696, v707
//     v709 = iconst.i32 7
//     v710 = imul v701, v709  ; v709 = 7
//     v711 = isub v699, v710
//     v712 = icmp slt v711, v541  ; v541 = 0
//     v713 = select v712, v542, v543  ; v542 = -1, v543 = 1
//     v714 = iconst.i32 14
//     v715 = sshr v708, v714  ; v714 = 14
//     v716 = imul v713, v715
//     v717 = isub v705, v716
//     v718 = sshr v705, v714  ; v714 = 14
//     v719 = imul v713, v718
//     v720 = iadd v708, v719
//     v721 = iconst.i32 3
//     v722 = imul v713, v721  ; v721 = 3
//     v723 = isub v711, v722
//     v724 = icmp slt v723, v541  ; v541 = 0
//     v725 = select v724, v542, v543  ; v542 = -1, v543 = 1
//     v726 = iconst.i32 15
//     v727 = sshr v720, v726  ; v726 = 15
//     v728 = imul v725, v727
//     v729 = isub v717, v728
//     v730 = sshr v717, v726  ; v726 = 15
//     v731 = imul v725, v730
//     v732 = iadd v720, v731
//     v733 = iconst.i32 1
//     v734 = imul v725, v733  ; v733 = 1
//     v735 = isub v723, v734
//     v736 = iconst.i32 0x0001_a592
//     v737 = sextend.i64 v732
//     v738 = sextend.i64 v736  ; v736 = 0x0001_a592
//     v739 = imul v737, v738
//     v740 = iconst.i64 16
//     v741 = sshr v739, v740  ; v740 = 16
//     v742 = ireduce.i32 v741
//     v743 = sextend.i64 v729
//     v744 = imul v743, v738
//     v745 = sshr v744, v740  ; v740 = 16
//     v746 = ireduce.i32 v745
//     v747 = iconst.i32 0
//     v748 = iconst.i32 1
//     v749 = iconst.i32 2
//     v750 = iconst.i32 3
//     v751 = icmp eq v538, v747  ; v747 = 0
//     v752 = icmp eq v538, v748  ; v748 = 1
//     v753 = icmp eq v538, v749  ; v749 = 2
//     v754 = icmp eq v538, v750  ; v750 = 3
//     v755 = ineg v742
//     v756 = select v753, v755, v742
//     v757 = select v754, v755, v756
//     v758 = select v752, v742, v757
//     v759 = select v751, v742, v758
//     v760 = iconst.i32 0x0003_243f
//     v761 = iconst.i32 0x0001_921f
//     v762 = iconst.i32 0x0006_487e
//     v763 = iconst.i32 0
//     v764 = iadd v760, v761  ; v760 = 0x0003_243f, v761 = 0x0001_921f
//     v765 = iconst.i32 1
//     v766 = iconst.i32 2
//     v767 = iconst.i32 3
//     v768 = icmp sle v11, v761  ; v11 = 0xc910, v761 = 0x0001_921f
//     v769 = icmp sle v11, v760  ; v11 = 0xc910, v760 = 0x0003_243f
//     v770 = icmp sle v11, v764  ; v11 = 0xc910
//     v771 = icmp sgt v11, v764  ; v11 = 0xc910
//     v772 = isub v760, v11  ; v760 = 0x0003_243f, v11 = 0xc910
//     v773 = isub v11, v760  ; v11 = 0xc910, v760 = 0x0003_243f
//     v774 = isub v762, v11  ; v762 = 0x0006_487e, v11 = 0xc910
//     v775 = select v769, v772, v11  ; v11 = 0xc910
//     v776 = select v770, v773, v775
//     v777 = select v771, v774, v776
//     v778 = bnot v768
//     v779 = band v769, v778
//     v780 = select v768, v763, v763  ; v763 = 0, v763 = 0
//     v781 = select v779, v765, v780  ; v765 = 1
//     v782 = bnot v769
//     v783 = band v770, v782
//     v784 = select v783, v766, v781  ; v766 = 2
//     v785 = select v771, v767, v784  ; v767 = 3
//     v786 = iconst.i32 0x9b74
//     v787 = iconst.i32 0
//     v788 = iconst.i32 0
//     v789 = iconst.i32 -1
//     v790 = iconst.i32 1
//     v791 = icmp slt v777, v788  ; v788 = 0
//     v792 = select v791, v789, v790  ; v789 = -1, v790 = 1
//     v793 = iconst.i32 0
//     v794 = sshr v787, v793  ; v787 = 0, v793 = 0
//     v795 = imul v792, v794
//     v796 = isub v786, v795  ; v786 = 0x9b74
//     v797 = sshr v786, v793  ; v786 = 0x9b74, v793 = 0
//     v798 = imul v792, v797
//     v799 = iadd v787, v798  ; v787 = 0
//     v800 = iconst.i32 0xc90f
//     v801 = imul v792, v800  ; v800 = 0xc90f
//     v802 = isub v777, v801
//     v803 = icmp slt v802, v788  ; v788 = 0
//     v804 = select v803, v789, v790  ; v789 = -1, v790 = 1
//     v805 = iconst.i32 1
//     v806 = sshr v799, v805  ; v805 = 1
//     v807 = imul v804, v806
//     v808 = isub v796, v807
//     v809 = sshr v796, v805  ; v805 = 1
//     v810 = imul v804, v809
//     v811 = iadd v799, v810
//     v812 = iconst.i32 0x76b1
//     v813 = imul v804, v812  ; v812 = 0x76b1
//     v814 = isub v802, v813
//     v815 = icmp slt v814, v788  ; v788 = 0
//     v816 = select v815, v789, v790  ; v789 = -1, v790 = 1
//     v817 = iconst.i32 2
//     v818 = sshr v811, v817  ; v817 = 2
//     v819 = imul v816, v818
//     v820 = isub v808, v819
//     v821 = sshr v808, v817  ; v817 = 2
//     v822 = imul v816, v821
//     v823 = iadd v811, v822
//     v824 = iconst.i32 0x3eb6
//     v825 = imul v816, v824  ; v824 = 0x3eb6
//     v826 = isub v814, v825
//     v827 = icmp slt v826, v788  ; v788 = 0
//     v828 = select v827, v789, v790  ; v789 = -1, v790 = 1
//     v829 = iconst.i32 3
//     v830 = sshr v823, v829  ; v829 = 3
//     v831 = imul v828, v830
//     v832 = isub v820, v831
//     v833 = sshr v820, v829  ; v829 = 3
//     v834 = imul v828, v833
//     v835 = iadd v823, v834
//     v836 = iconst.i32 8149
//     v837 = imul v828, v836  ; v836 = 8149
//     v838 = isub v826, v837
//     v839 = icmp slt v838, v788  ; v788 = 0
//     v840 = select v839, v789, v790  ; v789 = -1, v790 = 1
//     v841 = iconst.i32 4
//     v842 = sshr v835, v841  ; v841 = 4
//     v843 = imul v840, v842
//     v844 = isub v832, v843
//     v845 = sshr v832, v841  ; v841 = 4
//     v846 = imul v840, v845
//     v847 = iadd v835, v846
//     v848 = iconst.i32 4090
//     v849 = imul v840, v848  ; v848 = 4090
//     v850 = isub v838, v849
//     v851 = icmp slt v850, v788  ; v788 = 0
//     v852 = select v851, v789, v790  ; v789 = -1, v790 = 1
//     v853 = iconst.i32 5
//     v854 = sshr v847, v853  ; v853 = 5
//     v855 = imul v852, v854
//     v856 = isub v844, v855
//     v857 = sshr v844, v853  ; v853 = 5
//     v858 = imul v852, v857
//     v859 = iadd v847, v858
//     v860 = iconst.i32 2047
//     v861 = imul v852, v860  ; v860 = 2047
//     v862 = isub v850, v861
//     v863 = icmp slt v862, v788  ; v788 = 0
//     v864 = select v863, v789, v790  ; v789 = -1, v790 = 1
//     v865 = iconst.i32 6
//     v866 = sshr v859, v865  ; v865 = 6
//     v867 = imul v864, v866
//     v868 = isub v856, v867
//     v869 = sshr v856, v865  ; v865 = 6
//     v870 = imul v864, v869
//     v871 = iadd v859, v870
//     v872 = iconst.i32 1023
//     v873 = imul v864, v872  ; v872 = 1023
//     v874 = isub v862, v873
//     v875 = icmp slt v874, v788  ; v788 = 0
//     v876 = select v875, v789, v790  ; v789 = -1, v790 = 1
//     v877 = iconst.i32 7
//     v878 = sshr v871, v877  ; v877 = 7
//     v879 = imul v876, v878
//     v880 = isub v868, v879
//     v881 = sshr v868, v877  ; v877 = 7
//     v882 = imul v876, v881
//     v883 = iadd v871, v882
//     v884 = iconst.i32 511
//     v885 = imul v876, v884  ; v884 = 511
//     v886 = isub v874, v885
//     v887 = icmp slt v886, v788  ; v788 = 0
//     v888 = select v887, v789, v790  ; v789 = -1, v790 = 1
//     v889 = iconst.i32 8
//     v890 = sshr v883, v889  ; v889 = 8
//     v891 = imul v888, v890
//     v892 = isub v880, v891
//     v893 = sshr v880, v889  ; v889 = 8
//     v894 = imul v888, v893
//     v895 = iadd v883, v894
//     v896 = iconst.i32 255
//     v897 = imul v888, v896  ; v896 = 255
//     v898 = isub v886, v897
//     v899 = icmp slt v898, v788  ; v788 = 0
//     v900 = select v899, v789, v790  ; v789 = -1, v790 = 1
//     v901 = iconst.i32 9
//     v902 = sshr v895, v901  ; v901 = 9
//     v903 = imul v900, v902
//     v904 = isub v892, v903
//     v905 = sshr v892, v901  ; v901 = 9
//     v906 = imul v900, v905
//     v907 = iadd v895, v906
//     v908 = iconst.i32 127
//     v909 = imul v900, v908  ; v908 = 127
//     v910 = isub v898, v909
//     v911 = icmp slt v910, v788  ; v788 = 0
//     v912 = select v911, v789, v790  ; v789 = -1, v790 = 1
//     v913 = iconst.i32 10
//     v914 = sshr v907, v913  ; v913 = 10
//     v915 = imul v912, v914
//     v916 = isub v904, v915
//     v917 = sshr v904, v913  ; v913 = 10
//     v918 = imul v912, v917
//     v919 = iadd v907, v918
//     v920 = iconst.i32 63
//     v921 = imul v912, v920  ; v920 = 63
//     v922 = isub v910, v921
//     v923 = icmp slt v922, v788  ; v788 = 0
//     v924 = select v923, v789, v790  ; v789 = -1, v790 = 1
//     v925 = iconst.i32 11
//     v926 = sshr v919, v925  ; v925 = 11
//     v927 = imul v924, v926
//     v928 = isub v916, v927
//     v929 = sshr v916, v925  ; v925 = 11
//     v930 = imul v924, v929
//     v931 = iadd v919, v930
//     v932 = iconst.i32 31
//     v933 = imul v924, v932  ; v932 = 31
//     v934 = isub v922, v933
//     v935 = icmp slt v934, v788  ; v788 = 0
//     v936 = select v935, v789, v790  ; v789 = -1, v790 = 1
//     v937 = iconst.i32 12
//     v938 = sshr v931, v937  ; v937 = 12
//     v939 = imul v936, v938
//     v940 = isub v928, v939
//     v941 = sshr v928, v937  ; v937 = 12
//     v942 = imul v936, v941
//     v943 = iadd v931, v942
//     v944 = iconst.i32 15
//     v945 = imul v936, v944  ; v944 = 15
//     v946 = isub v934, v945
//     v947 = icmp slt v946, v788  ; v788 = 0
//     v948 = select v947, v789, v790  ; v789 = -1, v790 = 1
//     v949 = iconst.i32 13
//     v950 = sshr v943, v949  ; v949 = 13
//     v951 = imul v948, v950
//     v952 = isub v940, v951
//     v953 = sshr v940, v949  ; v949 = 13
//     v954 = imul v948, v953
//     v955 = iadd v943, v954
//     v956 = iconst.i32 7
//     v957 = imul v948, v956  ; v956 = 7
//     v958 = isub v946, v957
//     v959 = icmp slt v958, v788  ; v788 = 0
//     v960 = select v959, v789, v790  ; v789 = -1, v790 = 1
//     v961 = iconst.i32 14
//     v962 = sshr v955, v961  ; v961 = 14
//     v963 = imul v960, v962
//     v964 = isub v952, v963
//     v965 = sshr v952, v961  ; v961 = 14
//     v966 = imul v960, v965
//     v967 = iadd v955, v966
//     v968 = iconst.i32 3
//     v969 = imul v960, v968  ; v968 = 3
//     v970 = isub v958, v969
//     v971 = icmp slt v970, v788  ; v788 = 0
//     v972 = select v971, v789, v790  ; v789 = -1, v790 = 1
//     v973 = iconst.i32 15
//     v974 = sshr v967, v973  ; v973 = 15
//     v975 = imul v972, v974
//     v976 = isub v964, v975
//     v977 = sshr v964, v973  ; v973 = 15
//     v978 = imul v972, v977
//     v979 = iadd v967, v978
//     v980 = iconst.i32 1
//     v981 = imul v972, v980  ; v980 = 1
//     v982 = isub v970, v981
//     v983 = iconst.i32 0x0001_a592
//     v984 = sextend.i64 v979
//     v985 = sextend.i64 v983  ; v983 = 0x0001_a592
//     v986 = imul v984, v985
//     v987 = iconst.i64 16
//     v988 = sshr v986, v987  ; v987 = 16
//     v989 = ireduce.i32 v988
//     v990 = sextend.i64 v976
//     v991 = imul v990, v985
//     v992 = sshr v991, v987  ; v987 = 16
//     v993 = ireduce.i32 v992
//     v994 = iconst.i32 0
//     v995 = iconst.i32 1
//     v996 = iconst.i32 2
//     v997 = iconst.i32 3
//     v998 = icmp eq v785, v994  ; v994 = 0
//     v999 = icmp eq v785, v995  ; v995 = 1
//     v1000 = icmp eq v785, v996  ; v996 = 2
//     v1001 = icmp eq v785, v997  ; v997 = 3
//     v1002 = ineg v993
//     v1003 = select v999, v1002, v993
//     v1004 = select v1000, v1002, v1003
//     v1005 = select v1001, v993, v1004
//     v1006 = select v998, v993, v1005
//     v1007 = sextend.i64 v759
//     v1008 = iconst.i64 16
//     v1009 = ishl v1007, v1008  ; v1008 = 16
//     v1010 = sextend.i64 v1006
//     v1011 = sdiv v1009, v1010
//     v1012 = ireduce.i32 v1011
//     v1013 = iconst.i32 0x0003_243f
//     v1014 = iconst.i32 0x0001_921f
//     v1015 = iconst.i32 0x0006_487e
//     v1016 = iconst.i32 0
//     v1017 = iadd v1013, v1014  ; v1013 = 0x0003_243f, v1014 = 0x0001_921f
//     v1018 = iconst.i32 1
//     v1019 = iconst.i32 2
//     v1020 = iconst.i32 3
//     v1021 = icmp sle v12, v1014  ; v12 = 0x0001_9220, v1014 = 0x0001_921f
//     v1022 = icmp sle v12, v1013  ; v12 = 0x0001_9220, v1013 = 0x0003_243f
//     v1023 = icmp sle v12, v1017  ; v12 = 0x0001_9220
//     v1024 = icmp sgt v12, v1017  ; v12 = 0x0001_9220
//     v1025 = isub v1013, v12  ; v1013 = 0x0003_243f, v12 = 0x0001_9220
//     v1026 = isub v12, v1013  ; v12 = 0x0001_9220, v1013 = 0x0003_243f
//     v1027 = isub v1015, v12  ; v1015 = 0x0006_487e, v12 = 0x0001_9220
//     v1028 = select v1022, v1025, v12  ; v12 = 0x0001_9220
//     v1029 = select v1023, v1026, v1028
//     v1030 = select v1024, v1027, v1029
//     v1031 = bnot v1021
//     v1032 = band v1022, v1031
//     v1033 = select v1021, v1016, v1016  ; v1016 = 0, v1016 = 0
//     v1034 = select v1032, v1018, v1033  ; v1018 = 1
//     v1035 = bnot v1022
//     v1036 = band v1023, v1035
//     v1037 = select v1036, v1019, v1034  ; v1019 = 2
//     v1038 = select v1024, v1020, v1037  ; v1020 = 3
//     v1039 = iconst.i32 0x9b74
//     v1040 = iconst.i32 0
//     v1041 = iconst.i32 0
//     v1042 = iconst.i32 -1
//     v1043 = iconst.i32 1
//     v1044 = icmp slt v1030, v1041  ; v1041 = 0
//     v1045 = select v1044, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1046 = iconst.i32 0
//     v1047 = sshr v1040, v1046  ; v1040 = 0, v1046 = 0
//     v1048 = imul v1045, v1047
//     v1049 = isub v1039, v1048  ; v1039 = 0x9b74
//     v1050 = sshr v1039, v1046  ; v1039 = 0x9b74, v1046 = 0
//     v1051 = imul v1045, v1050
//     v1052 = iadd v1040, v1051  ; v1040 = 0
//     v1053 = iconst.i32 0xc90f
//     v1054 = imul v1045, v1053  ; v1053 = 0xc90f
//     v1055 = isub v1030, v1054
//     v1056 = icmp slt v1055, v1041  ; v1041 = 0
//     v1057 = select v1056, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1058 = iconst.i32 1
//     v1059 = sshr v1052, v1058  ; v1058 = 1
//     v1060 = imul v1057, v1059
//     v1061 = isub v1049, v1060
//     v1062 = sshr v1049, v1058  ; v1058 = 1
//     v1063 = imul v1057, v1062
//     v1064 = iadd v1052, v1063
//     v1065 = iconst.i32 0x76b1
//     v1066 = imul v1057, v1065  ; v1065 = 0x76b1
//     v1067 = isub v1055, v1066
//     v1068 = icmp slt v1067, v1041  ; v1041 = 0
//     v1069 = select v1068, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1070 = iconst.i32 2
//     v1071 = sshr v1064, v1070  ; v1070 = 2
//     v1072 = imul v1069, v1071
//     v1073 = isub v1061, v1072
//     v1074 = sshr v1061, v1070  ; v1070 = 2
//     v1075 = imul v1069, v1074
//     v1076 = iadd v1064, v1075
//     v1077 = iconst.i32 0x3eb6
//     v1078 = imul v1069, v1077  ; v1077 = 0x3eb6
//     v1079 = isub v1067, v1078
//     v1080 = icmp slt v1079, v1041  ; v1041 = 0
//     v1081 = select v1080, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1082 = iconst.i32 3
//     v1083 = sshr v1076, v1082  ; v1082 = 3
//     v1084 = imul v1081, v1083
//     v1085 = isub v1073, v1084
//     v1086 = sshr v1073, v1082  ; v1082 = 3
//     v1087 = imul v1081, v1086
//     v1088 = iadd v1076, v1087
//     v1089 = iconst.i32 8149
//     v1090 = imul v1081, v1089  ; v1089 = 8149
//     v1091 = isub v1079, v1090
//     v1092 = icmp slt v1091, v1041  ; v1041 = 0
//     v1093 = select v1092, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1094 = iconst.i32 4
//     v1095 = sshr v1088, v1094  ; v1094 = 4
//     v1096 = imul v1093, v1095
//     v1097 = isub v1085, v1096
//     v1098 = sshr v1085, v1094  ; v1094 = 4
//     v1099 = imul v1093, v1098
//     v1100 = iadd v1088, v1099
//     v1101 = iconst.i32 4090
//     v1102 = imul v1093, v1101  ; v1101 = 4090
//     v1103 = isub v1091, v1102
//     v1104 = icmp slt v1103, v1041  ; v1041 = 0
//     v1105 = select v1104, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1106 = iconst.i32 5
//     v1107 = sshr v1100, v1106  ; v1106 = 5
//     v1108 = imul v1105, v1107
//     v1109 = isub v1097, v1108
//     v1110 = sshr v1097, v1106  ; v1106 = 5
//     v1111 = imul v1105, v1110
//     v1112 = iadd v1100, v1111
//     v1113 = iconst.i32 2047
//     v1114 = imul v1105, v1113  ; v1113 = 2047
//     v1115 = isub v1103, v1114
//     v1116 = icmp slt v1115, v1041  ; v1041 = 0
//     v1117 = select v1116, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1118 = iconst.i32 6
//     v1119 = sshr v1112, v1118  ; v1118 = 6
//     v1120 = imul v1117, v1119
//     v1121 = isub v1109, v1120
//     v1122 = sshr v1109, v1118  ; v1118 = 6
//     v1123 = imul v1117, v1122
//     v1124 = iadd v1112, v1123
//     v1125 = iconst.i32 1023
//     v1126 = imul v1117, v1125  ; v1125 = 1023
//     v1127 = isub v1115, v1126
//     v1128 = icmp slt v1127, v1041  ; v1041 = 0
//     v1129 = select v1128, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1130 = iconst.i32 7
//     v1131 = sshr v1124, v1130  ; v1130 = 7
//     v1132 = imul v1129, v1131
//     v1133 = isub v1121, v1132
//     v1134 = sshr v1121, v1130  ; v1130 = 7
//     v1135 = imul v1129, v1134
//     v1136 = iadd v1124, v1135
//     v1137 = iconst.i32 511
//     v1138 = imul v1129, v1137  ; v1137 = 511
//     v1139 = isub v1127, v1138
//     v1140 = icmp slt v1139, v1041  ; v1041 = 0
//     v1141 = select v1140, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1142 = iconst.i32 8
//     v1143 = sshr v1136, v1142  ; v1142 = 8
//     v1144 = imul v1141, v1143
//     v1145 = isub v1133, v1144
//     v1146 = sshr v1133, v1142  ; v1142 = 8
//     v1147 = imul v1141, v1146
//     v1148 = iadd v1136, v1147
//     v1149 = iconst.i32 255
//     v1150 = imul v1141, v1149  ; v1149 = 255
//     v1151 = isub v1139, v1150
//     v1152 = icmp slt v1151, v1041  ; v1041 = 0
//     v1153 = select v1152, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1154 = iconst.i32 9
//     v1155 = sshr v1148, v1154  ; v1154 = 9
//     v1156 = imul v1153, v1155
//     v1157 = isub v1145, v1156
//     v1158 = sshr v1145, v1154  ; v1154 = 9
//     v1159 = imul v1153, v1158
//     v1160 = iadd v1148, v1159
//     v1161 = iconst.i32 127
//     v1162 = imul v1153, v1161  ; v1161 = 127
//     v1163 = isub v1151, v1162
//     v1164 = icmp slt v1163, v1041  ; v1041 = 0
//     v1165 = select v1164, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1166 = iconst.i32 10
//     v1167 = sshr v1160, v1166  ; v1166 = 10
//     v1168 = imul v1165, v1167
//     v1169 = isub v1157, v1168
//     v1170 = sshr v1157, v1166  ; v1166 = 10
//     v1171 = imul v1165, v1170
//     v1172 = iadd v1160, v1171
//     v1173 = iconst.i32 63
//     v1174 = imul v1165, v1173  ; v1173 = 63
//     v1175 = isub v1163, v1174
//     v1176 = icmp slt v1175, v1041  ; v1041 = 0
//     v1177 = select v1176, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1178 = iconst.i32 11
//     v1179 = sshr v1172, v1178  ; v1178 = 11
//     v1180 = imul v1177, v1179
//     v1181 = isub v1169, v1180
//     v1182 = sshr v1169, v1178  ; v1178 = 11
//     v1183 = imul v1177, v1182
//     v1184 = iadd v1172, v1183
//     v1185 = iconst.i32 31
//     v1186 = imul v1177, v1185  ; v1185 = 31
//     v1187 = isub v1175, v1186
//     v1188 = icmp slt v1187, v1041  ; v1041 = 0
//     v1189 = select v1188, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1190 = iconst.i32 12
//     v1191 = sshr v1184, v1190  ; v1190 = 12
//     v1192 = imul v1189, v1191
//     v1193 = isub v1181, v1192
//     v1194 = sshr v1181, v1190  ; v1190 = 12
//     v1195 = imul v1189, v1194
//     v1196 = iadd v1184, v1195
//     v1197 = iconst.i32 15
//     v1198 = imul v1189, v1197  ; v1197 = 15
//     v1199 = isub v1187, v1198
//     v1200 = icmp slt v1199, v1041  ; v1041 = 0
//     v1201 = select v1200, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1202 = iconst.i32 13
//     v1203 = sshr v1196, v1202  ; v1202 = 13
//     v1204 = imul v1201, v1203
//     v1205 = isub v1193, v1204
//     v1206 = sshr v1193, v1202  ; v1202 = 13
//     v1207 = imul v1201, v1206
//     v1208 = iadd v1196, v1207
//     v1209 = iconst.i32 7
//     v1210 = imul v1201, v1209  ; v1209 = 7
//     v1211 = isub v1199, v1210
//     v1212 = icmp slt v1211, v1041  ; v1041 = 0
//     v1213 = select v1212, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1214 = iconst.i32 14
//     v1215 = sshr v1208, v1214  ; v1214 = 14
//     v1216 = imul v1213, v1215
//     v1217 = isub v1205, v1216
//     v1218 = sshr v1205, v1214  ; v1214 = 14
//     v1219 = imul v1213, v1218
//     v1220 = iadd v1208, v1219
//     v1221 = iconst.i32 3
//     v1222 = imul v1213, v1221  ; v1221 = 3
//     v1223 = isub v1211, v1222
//     v1224 = icmp slt v1223, v1041  ; v1041 = 0
//     v1225 = select v1224, v1042, v1043  ; v1042 = -1, v1043 = 1
//     v1226 = iconst.i32 15
//     v1227 = sshr v1220, v1226  ; v1226 = 15
//     v1228 = imul v1225, v1227
//     v1229 = isub v1217, v1228
//     v1230 = sshr v1217, v1226  ; v1226 = 15
//     v1231 = imul v1225, v1230
//     v1232 = iadd v1220, v1231
//     v1233 = iconst.i32 1
//     v1234 = imul v1225, v1233  ; v1233 = 1
//     v1235 = isub v1223, v1234
//     v1236 = iconst.i32 0x0001_a592
//     v1237 = sextend.i64 v1232
//     v1238 = sextend.i64 v1236  ; v1236 = 0x0001_a592
//     v1239 = imul v1237, v1238
//     v1240 = iconst.i64 16
//     v1241 = sshr v1239, v1240  ; v1240 = 16
//     v1242 = ireduce.i32 v1241
//     v1243 = sextend.i64 v1229
//     v1244 = imul v1243, v1238
//     v1245 = sshr v1244, v1240  ; v1240 = 16
//     v1246 = ireduce.i32 v1245
//     v1247 = iconst.i32 0
//     v1248 = iconst.i32 1
//     v1249 = iconst.i32 2
//     v1250 = iconst.i32 3
//     v1251 = icmp eq v1038, v1247  ; v1247 = 0
//     v1252 = icmp eq v1038, v1248  ; v1248 = 1
//     v1253 = icmp eq v1038, v1249  ; v1249 = 2
//     v1254 = icmp eq v1038, v1250  ; v1250 = 3
//     v1255 = ineg v1242
//     v1256 = select v1253, v1255, v1242
//     v1257 = select v1254, v1255, v1256
//     v1258 = select v1252, v1242, v1257
//     v1259 = select v1251, v1242, v1258
//     v1260 = iconst.i32 0x0003_243f
//     v1261 = iconst.i32 0x0001_921f
//     v1262 = iconst.i32 0x0006_487e
//     v1263 = iconst.i32 0
//     v1264 = iadd v1260, v1261  ; v1260 = 0x0003_243f, v1261 = 0x0001_921f
//     v1265 = iconst.i32 1
//     v1266 = iconst.i32 2
//     v1267 = iconst.i32 3
//     v1268 = icmp sle v12, v1261  ; v12 = 0x0001_9220, v1261 = 0x0001_921f
//     v1269 = icmp sle v12, v1260  ; v12 = 0x0001_9220, v1260 = 0x0003_243f
//     v1270 = icmp sle v12, v1264  ; v12 = 0x0001_9220
//     v1271 = icmp sgt v12, v1264  ; v12 = 0x0001_9220
//     v1272 = isub v1260, v12  ; v1260 = 0x0003_243f, v12 = 0x0001_9220
//     v1273 = isub v12, v1260  ; v12 = 0x0001_9220, v1260 = 0x0003_243f
//     v1274 = isub v1262, v12  ; v1262 = 0x0006_487e, v12 = 0x0001_9220
//     v1275 = select v1269, v1272, v12  ; v12 = 0x0001_9220
//     v1276 = select v1270, v1273, v1275
//     v1277 = select v1271, v1274, v1276
//     v1278 = bnot v1268
//     v1279 = band v1269, v1278
//     v1280 = select v1268, v1263, v1263  ; v1263 = 0, v1263 = 0
//     v1281 = select v1279, v1265, v1280  ; v1265 = 1
//     v1282 = bnot v1269
//     v1283 = band v1270, v1282
//     v1284 = select v1283, v1266, v1281  ; v1266 = 2
//     v1285 = select v1271, v1267, v1284  ; v1267 = 3
//     v1286 = iconst.i32 0x9b74
//     v1287 = iconst.i32 0
//     v1288 = iconst.i32 0
//     v1289 = iconst.i32 -1
//     v1290 = iconst.i32 1
//     v1291 = icmp slt v1277, v1288  ; v1288 = 0
//     v1292 = select v1291, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1293 = iconst.i32 0
//     v1294 = sshr v1287, v1293  ; v1287 = 0, v1293 = 0
//     v1295 = imul v1292, v1294
//     v1296 = isub v1286, v1295  ; v1286 = 0x9b74
//     v1297 = sshr v1286, v1293  ; v1286 = 0x9b74, v1293 = 0
//     v1298 = imul v1292, v1297
//     v1299 = iadd v1287, v1298  ; v1287 = 0
//     v1300 = iconst.i32 0xc90f
//     v1301 = imul v1292, v1300  ; v1300 = 0xc90f
//     v1302 = isub v1277, v1301
//     v1303 = icmp slt v1302, v1288  ; v1288 = 0
//     v1304 = select v1303, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1305 = iconst.i32 1
//     v1306 = sshr v1299, v1305  ; v1305 = 1
//     v1307 = imul v1304, v1306
//     v1308 = isub v1296, v1307
//     v1309 = sshr v1296, v1305  ; v1305 = 1
//     v1310 = imul v1304, v1309
//     v1311 = iadd v1299, v1310
//     v1312 = iconst.i32 0x76b1
//     v1313 = imul v1304, v1312  ; v1312 = 0x76b1
//     v1314 = isub v1302, v1313
//     v1315 = icmp slt v1314, v1288  ; v1288 = 0
//     v1316 = select v1315, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1317 = iconst.i32 2
//     v1318 = sshr v1311, v1317  ; v1317 = 2
//     v1319 = imul v1316, v1318
//     v1320 = isub v1308, v1319
//     v1321 = sshr v1308, v1317  ; v1317 = 2
//     v1322 = imul v1316, v1321
//     v1323 = iadd v1311, v1322
//     v1324 = iconst.i32 0x3eb6
//     v1325 = imul v1316, v1324  ; v1324 = 0x3eb6
//     v1326 = isub v1314, v1325
//     v1327 = icmp slt v1326, v1288  ; v1288 = 0
//     v1328 = select v1327, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1329 = iconst.i32 3
//     v1330 = sshr v1323, v1329  ; v1329 = 3
//     v1331 = imul v1328, v1330
//     v1332 = isub v1320, v1331
//     v1333 = sshr v1320, v1329  ; v1329 = 3
//     v1334 = imul v1328, v1333
//     v1335 = iadd v1323, v1334
//     v1336 = iconst.i32 8149
//     v1337 = imul v1328, v1336  ; v1336 = 8149
//     v1338 = isub v1326, v1337
//     v1339 = icmp slt v1338, v1288  ; v1288 = 0
//     v1340 = select v1339, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1341 = iconst.i32 4
//     v1342 = sshr v1335, v1341  ; v1341 = 4
//     v1343 = imul v1340, v1342
//     v1344 = isub v1332, v1343
//     v1345 = sshr v1332, v1341  ; v1341 = 4
//     v1346 = imul v1340, v1345
//     v1347 = iadd v1335, v1346
//     v1348 = iconst.i32 4090
//     v1349 = imul v1340, v1348  ; v1348 = 4090
//     v1350 = isub v1338, v1349
//     v1351 = icmp slt v1350, v1288  ; v1288 = 0
//     v1352 = select v1351, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1353 = iconst.i32 5
//     v1354 = sshr v1347, v1353  ; v1353 = 5
//     v1355 = imul v1352, v1354
//     v1356 = isub v1344, v1355
//     v1357 = sshr v1344, v1353  ; v1353 = 5
//     v1358 = imul v1352, v1357
//     v1359 = iadd v1347, v1358
//     v1360 = iconst.i32 2047
//     v1361 = imul v1352, v1360  ; v1360 = 2047
//     v1362 = isub v1350, v1361
//     v1363 = icmp slt v1362, v1288  ; v1288 = 0
//     v1364 = select v1363, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1365 = iconst.i32 6
//     v1366 = sshr v1359, v1365  ; v1365 = 6
//     v1367 = imul v1364, v1366
//     v1368 = isub v1356, v1367
//     v1369 = sshr v1356, v1365  ; v1365 = 6
//     v1370 = imul v1364, v1369
//     v1371 = iadd v1359, v1370
//     v1372 = iconst.i32 1023
//     v1373 = imul v1364, v1372  ; v1372 = 1023
//     v1374 = isub v1362, v1373
//     v1375 = icmp slt v1374, v1288  ; v1288 = 0
//     v1376 = select v1375, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1377 = iconst.i32 7
//     v1378 = sshr v1371, v1377  ; v1377 = 7
//     v1379 = imul v1376, v1378
//     v1380 = isub v1368, v1379
//     v1381 = sshr v1368, v1377  ; v1377 = 7
//     v1382 = imul v1376, v1381
//     v1383 = iadd v1371, v1382
//     v1384 = iconst.i32 511
//     v1385 = imul v1376, v1384  ; v1384 = 511
//     v1386 = isub v1374, v1385
//     v1387 = icmp slt v1386, v1288  ; v1288 = 0
//     v1388 = select v1387, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1389 = iconst.i32 8
//     v1390 = sshr v1383, v1389  ; v1389 = 8
//     v1391 = imul v1388, v1390
//     v1392 = isub v1380, v1391
//     v1393 = sshr v1380, v1389  ; v1389 = 8
//     v1394 = imul v1388, v1393
//     v1395 = iadd v1383, v1394
//     v1396 = iconst.i32 255
//     v1397 = imul v1388, v1396  ; v1396 = 255
//     v1398 = isub v1386, v1397
//     v1399 = icmp slt v1398, v1288  ; v1288 = 0
//     v1400 = select v1399, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1401 = iconst.i32 9
//     v1402 = sshr v1395, v1401  ; v1401 = 9
//     v1403 = imul v1400, v1402
//     v1404 = isub v1392, v1403
//     v1405 = sshr v1392, v1401  ; v1401 = 9
//     v1406 = imul v1400, v1405
//     v1407 = iadd v1395, v1406
//     v1408 = iconst.i32 127
//     v1409 = imul v1400, v1408  ; v1408 = 127
//     v1410 = isub v1398, v1409
//     v1411 = icmp slt v1410, v1288  ; v1288 = 0
//     v1412 = select v1411, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1413 = iconst.i32 10
//     v1414 = sshr v1407, v1413  ; v1413 = 10
//     v1415 = imul v1412, v1414
//     v1416 = isub v1404, v1415
//     v1417 = sshr v1404, v1413  ; v1413 = 10
//     v1418 = imul v1412, v1417
//     v1419 = iadd v1407, v1418
//     v1420 = iconst.i32 63
//     v1421 = imul v1412, v1420  ; v1420 = 63
//     v1422 = isub v1410, v1421
//     v1423 = icmp slt v1422, v1288  ; v1288 = 0
//     v1424 = select v1423, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1425 = iconst.i32 11
//     v1426 = sshr v1419, v1425  ; v1425 = 11
//     v1427 = imul v1424, v1426
//     v1428 = isub v1416, v1427
//     v1429 = sshr v1416, v1425  ; v1425 = 11
//     v1430 = imul v1424, v1429
//     v1431 = iadd v1419, v1430
//     v1432 = iconst.i32 31
//     v1433 = imul v1424, v1432  ; v1432 = 31
//     v1434 = isub v1422, v1433
//     v1435 = icmp slt v1434, v1288  ; v1288 = 0
//     v1436 = select v1435, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1437 = iconst.i32 12
//     v1438 = sshr v1431, v1437  ; v1437 = 12
//     v1439 = imul v1436, v1438
//     v1440 = isub v1428, v1439
//     v1441 = sshr v1428, v1437  ; v1437 = 12
//     v1442 = imul v1436, v1441
//     v1443 = iadd v1431, v1442
//     v1444 = iconst.i32 15
//     v1445 = imul v1436, v1444  ; v1444 = 15
//     v1446 = isub v1434, v1445
//     v1447 = icmp slt v1446, v1288  ; v1288 = 0
//     v1448 = select v1447, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1449 = iconst.i32 13
//     v1450 = sshr v1443, v1449  ; v1449 = 13
//     v1451 = imul v1448, v1450
//     v1452 = isub v1440, v1451
//     v1453 = sshr v1440, v1449  ; v1449 = 13
//     v1454 = imul v1448, v1453
//     v1455 = iadd v1443, v1454
//     v1456 = iconst.i32 7
//     v1457 = imul v1448, v1456  ; v1456 = 7
//     v1458 = isub v1446, v1457
//     v1459 = icmp slt v1458, v1288  ; v1288 = 0
//     v1460 = select v1459, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1461 = iconst.i32 14
//     v1462 = sshr v1455, v1461  ; v1461 = 14
//     v1463 = imul v1460, v1462
//     v1464 = isub v1452, v1463
//     v1465 = sshr v1452, v1461  ; v1461 = 14
//     v1466 = imul v1460, v1465
//     v1467 = iadd v1455, v1466
//     v1468 = iconst.i32 3
//     v1469 = imul v1460, v1468  ; v1468 = 3
//     v1470 = isub v1458, v1469
//     v1471 = icmp slt v1470, v1288  ; v1288 = 0
//     v1472 = select v1471, v1289, v1290  ; v1289 = -1, v1290 = 1
//     v1473 = iconst.i32 15
//     v1474 = sshr v1467, v1473  ; v1473 = 15
//     v1475 = imul v1472, v1474
//     v1476 = isub v1464, v1475
//     v1477 = sshr v1464, v1473  ; v1473 = 15
//     v1478 = imul v1472, v1477
//     v1479 = iadd v1467, v1478
//     v1480 = iconst.i32 1
//     v1481 = imul v1472, v1480  ; v1480 = 1
//     v1482 = isub v1470, v1481
//     v1483 = iconst.i32 0x0001_a592
//     v1484 = sextend.i64 v1479
//     v1485 = sextend.i64 v1483  ; v1483 = 0x0001_a592
//     v1486 = imul v1484, v1485
//     v1487 = iconst.i64 16
//     v1488 = sshr v1486, v1487  ; v1487 = 16
//     v1489 = ireduce.i32 v1488
//     v1490 = sextend.i64 v1476
//     v1491 = imul v1490, v1485
//     v1492 = sshr v1491, v1487  ; v1487 = 16
//     v1493 = ireduce.i32 v1492
//     v1494 = iconst.i32 0
//     v1495 = iconst.i32 1
//     v1496 = iconst.i32 2
//     v1497 = iconst.i32 3
//     v1498 = icmp eq v1285, v1494  ; v1494 = 0
//     v1499 = icmp eq v1285, v1495  ; v1495 = 1
//     v1500 = icmp eq v1285, v1496  ; v1496 = 2
//     v1501 = icmp eq v1285, v1497  ; v1497 = 3
//     v1502 = ineg v1493
//     v1503 = select v1499, v1502, v1493
//     v1504 = select v1500, v1502, v1503
//     v1505 = select v1501, v1493, v1504
//     v1506 = select v1498, v1493, v1505
//     v1507 = sextend.i64 v1259
//     v1508 = iconst.i64 16
//     v1509 = ishl v1507, v1508  ; v1508 = 16
//     v1510 = sextend.i64 v1506
//     v1511 = sdiv v1509, v1510
//     v1512 = ireduce.i32 v1511
//     store notrap aligned v512, v0
//     store notrap aligned v1012, v0+4
//     store notrap aligned v1512, v0+8
//     return
//
// block1:
//     v1513 = iconst.i32 0
//     store notrap aligned v1513, v0  ; v1513 = 0
//     v1514 = iconst.i32 0
//     store notrap aligned v1514, v0+4  ; v1514 = 0
//     v1515 = iconst.i32 0
//     store notrap aligned v1515, v0+8  ; v1515 = 0
//     return
// }
// run: ≈ vec3(0, 1, -22877332) (tolerance: 0.1)
