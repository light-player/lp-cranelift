// test compile
// test run
// target riscv32.fixed32

vec3 main() {
    vec3 angles = vec3(0.0, 1.570796327, 3.141592654); // 0, π/2, π
    return sin(angles);
}

// function u0:0(i32 sret) system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = %sinf sig0
//
// block0(v0: i32):
//     v10 = iconst.i32 0
//     v11 = iconst.i32 0x0001_9220
//     v12 = iconst.i32 0x0003_243f
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
//     v268 = icmp sle v11, v261  ; v11 = 0x0001_9220, v261 = 0x0001_921f
//     v269 = icmp sle v11, v260  ; v11 = 0x0001_9220, v260 = 0x0003_243f
//     v270 = icmp sle v11, v264  ; v11 = 0x0001_9220
//     v271 = icmp sgt v11, v264  ; v11 = 0x0001_9220
//     v272 = isub v260, v11  ; v260 = 0x0003_243f, v11 = 0x0001_9220
//     v273 = isub v11, v260  ; v11 = 0x0001_9220, v260 = 0x0003_243f
//     v274 = isub v262, v11  ; v262 = 0x0006_487e, v11 = 0x0001_9220
//     v275 = select v269, v272, v11  ; v11 = 0x0001_9220
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
//     v502 = ineg v489
//     v503 = select v500, v502, v489
//     v504 = select v501, v502, v503
//     v505 = select v499, v489, v504
//     v506 = select v498, v489, v505
//     v507 = iconst.i32 0x0003_243f
//     v508 = iconst.i32 0x0001_921f
//     v509 = iconst.i32 0x0006_487e
//     v510 = iconst.i32 0
//     v511 = iadd v507, v508  ; v507 = 0x0003_243f, v508 = 0x0001_921f
//     v512 = iconst.i32 1
//     v513 = iconst.i32 2
//     v514 = iconst.i32 3
//     v515 = icmp sle v12, v508  ; v12 = 0x0003_243f, v508 = 0x0001_921f
//     v516 = icmp sle v12, v507  ; v12 = 0x0003_243f, v507 = 0x0003_243f
//     v517 = icmp sle v12, v511  ; v12 = 0x0003_243f
//     v518 = icmp sgt v12, v511  ; v12 = 0x0003_243f
//     v519 = isub v507, v12  ; v507 = 0x0003_243f, v12 = 0x0003_243f
//     v520 = isub v12, v507  ; v12 = 0x0003_243f, v507 = 0x0003_243f
//     v521 = isub v509, v12  ; v509 = 0x0006_487e, v12 = 0x0003_243f
//     v522 = select v516, v519, v12  ; v12 = 0x0003_243f
//     v523 = select v517, v520, v522
//     v524 = select v518, v521, v523
//     v525 = bnot v515
//     v526 = band v516, v525
//     v527 = select v515, v510, v510  ; v510 = 0, v510 = 0
//     v528 = select v526, v512, v527  ; v512 = 1
//     v529 = bnot v516
//     v530 = band v517, v529
//     v531 = select v530, v513, v528  ; v513 = 2
//     v532 = select v518, v514, v531  ; v514 = 3
//     v533 = iconst.i32 0x9b74
//     v534 = iconst.i32 0
//     v535 = iconst.i32 0
//     v536 = iconst.i32 -1
//     v537 = iconst.i32 1
//     v538 = icmp slt v524, v535  ; v535 = 0
//     v539 = select v538, v536, v537  ; v536 = -1, v537 = 1
//     v540 = iconst.i32 0
//     v541 = sshr v534, v540  ; v534 = 0, v540 = 0
//     v542 = imul v539, v541
//     v543 = isub v533, v542  ; v533 = 0x9b74
//     v544 = sshr v533, v540  ; v533 = 0x9b74, v540 = 0
//     v545 = imul v539, v544
//     v546 = iadd v534, v545  ; v534 = 0
//     v547 = iconst.i32 0xc90f
//     v548 = imul v539, v547  ; v547 = 0xc90f
//     v549 = isub v524, v548
//     v550 = icmp slt v549, v535  ; v535 = 0
//     v551 = select v550, v536, v537  ; v536 = -1, v537 = 1
//     v552 = iconst.i32 1
//     v553 = sshr v546, v552  ; v552 = 1
//     v554 = imul v551, v553
//     v555 = isub v543, v554
//     v556 = sshr v543, v552  ; v552 = 1
//     v557 = imul v551, v556
//     v558 = iadd v546, v557
//     v559 = iconst.i32 0x76b1
//     v560 = imul v551, v559  ; v559 = 0x76b1
//     v561 = isub v549, v560
//     v562 = icmp slt v561, v535  ; v535 = 0
//     v563 = select v562, v536, v537  ; v536 = -1, v537 = 1
//     v564 = iconst.i32 2
//     v565 = sshr v558, v564  ; v564 = 2
//     v566 = imul v563, v565
//     v567 = isub v555, v566
//     v568 = sshr v555, v564  ; v564 = 2
//     v569 = imul v563, v568
//     v570 = iadd v558, v569
//     v571 = iconst.i32 0x3eb6
//     v572 = imul v563, v571  ; v571 = 0x3eb6
//     v573 = isub v561, v572
//     v574 = icmp slt v573, v535  ; v535 = 0
//     v575 = select v574, v536, v537  ; v536 = -1, v537 = 1
//     v576 = iconst.i32 3
//     v577 = sshr v570, v576  ; v576 = 3
//     v578 = imul v575, v577
//     v579 = isub v567, v578
//     v580 = sshr v567, v576  ; v576 = 3
//     v581 = imul v575, v580
//     v582 = iadd v570, v581
//     v583 = iconst.i32 8149
//     v584 = imul v575, v583  ; v583 = 8149
//     v585 = isub v573, v584
//     v586 = icmp slt v585, v535  ; v535 = 0
//     v587 = select v586, v536, v537  ; v536 = -1, v537 = 1
//     v588 = iconst.i32 4
//     v589 = sshr v582, v588  ; v588 = 4
//     v590 = imul v587, v589
//     v591 = isub v579, v590
//     v592 = sshr v579, v588  ; v588 = 4
//     v593 = imul v587, v592
//     v594 = iadd v582, v593
//     v595 = iconst.i32 4090
//     v596 = imul v587, v595  ; v595 = 4090
//     v597 = isub v585, v596
//     v598 = icmp slt v597, v535  ; v535 = 0
//     v599 = select v598, v536, v537  ; v536 = -1, v537 = 1
//     v600 = iconst.i32 5
//     v601 = sshr v594, v600  ; v600 = 5
//     v602 = imul v599, v601
//     v603 = isub v591, v602
//     v604 = sshr v591, v600  ; v600 = 5
//     v605 = imul v599, v604
//     v606 = iadd v594, v605
//     v607 = iconst.i32 2047
//     v608 = imul v599, v607  ; v607 = 2047
//     v609 = isub v597, v608
//     v610 = icmp slt v609, v535  ; v535 = 0
//     v611 = select v610, v536, v537  ; v536 = -1, v537 = 1
//     v612 = iconst.i32 6
//     v613 = sshr v606, v612  ; v612 = 6
//     v614 = imul v611, v613
//     v615 = isub v603, v614
//     v616 = sshr v603, v612  ; v612 = 6
//     v617 = imul v611, v616
//     v618 = iadd v606, v617
//     v619 = iconst.i32 1023
//     v620 = imul v611, v619  ; v619 = 1023
//     v621 = isub v609, v620
//     v622 = icmp slt v621, v535  ; v535 = 0
//     v623 = select v622, v536, v537  ; v536 = -1, v537 = 1
//     v624 = iconst.i32 7
//     v625 = sshr v618, v624  ; v624 = 7
//     v626 = imul v623, v625
//     v627 = isub v615, v626
//     v628 = sshr v615, v624  ; v624 = 7
//     v629 = imul v623, v628
//     v630 = iadd v618, v629
//     v631 = iconst.i32 511
//     v632 = imul v623, v631  ; v631 = 511
//     v633 = isub v621, v632
//     v634 = icmp slt v633, v535  ; v535 = 0
//     v635 = select v634, v536, v537  ; v536 = -1, v537 = 1
//     v636 = iconst.i32 8
//     v637 = sshr v630, v636  ; v636 = 8
//     v638 = imul v635, v637
//     v639 = isub v627, v638
//     v640 = sshr v627, v636  ; v636 = 8
//     v641 = imul v635, v640
//     v642 = iadd v630, v641
//     v643 = iconst.i32 255
//     v644 = imul v635, v643  ; v643 = 255
//     v645 = isub v633, v644
//     v646 = icmp slt v645, v535  ; v535 = 0
//     v647 = select v646, v536, v537  ; v536 = -1, v537 = 1
//     v648 = iconst.i32 9
//     v649 = sshr v642, v648  ; v648 = 9
//     v650 = imul v647, v649
//     v651 = isub v639, v650
//     v652 = sshr v639, v648  ; v648 = 9
//     v653 = imul v647, v652
//     v654 = iadd v642, v653
//     v655 = iconst.i32 127
//     v656 = imul v647, v655  ; v655 = 127
//     v657 = isub v645, v656
//     v658 = icmp slt v657, v535  ; v535 = 0
//     v659 = select v658, v536, v537  ; v536 = -1, v537 = 1
//     v660 = iconst.i32 10
//     v661 = sshr v654, v660  ; v660 = 10
//     v662 = imul v659, v661
//     v663 = isub v651, v662
//     v664 = sshr v651, v660  ; v660 = 10
//     v665 = imul v659, v664
//     v666 = iadd v654, v665
//     v667 = iconst.i32 63
//     v668 = imul v659, v667  ; v667 = 63
//     v669 = isub v657, v668
//     v670 = icmp slt v669, v535  ; v535 = 0
//     v671 = select v670, v536, v537  ; v536 = -1, v537 = 1
//     v672 = iconst.i32 11
//     v673 = sshr v666, v672  ; v672 = 11
//     v674 = imul v671, v673
//     v675 = isub v663, v674
//     v676 = sshr v663, v672  ; v672 = 11
//     v677 = imul v671, v676
//     v678 = iadd v666, v677
//     v679 = iconst.i32 31
//     v680 = imul v671, v679  ; v679 = 31
//     v681 = isub v669, v680
//     v682 = icmp slt v681, v535  ; v535 = 0
//     v683 = select v682, v536, v537  ; v536 = -1, v537 = 1
//     v684 = iconst.i32 12
//     v685 = sshr v678, v684  ; v684 = 12
//     v686 = imul v683, v685
//     v687 = isub v675, v686
//     v688 = sshr v675, v684  ; v684 = 12
//     v689 = imul v683, v688
//     v690 = iadd v678, v689
//     v691 = iconst.i32 15
//     v692 = imul v683, v691  ; v691 = 15
//     v693 = isub v681, v692
//     v694 = icmp slt v693, v535  ; v535 = 0
//     v695 = select v694, v536, v537  ; v536 = -1, v537 = 1
//     v696 = iconst.i32 13
//     v697 = sshr v690, v696  ; v696 = 13
//     v698 = imul v695, v697
//     v699 = isub v687, v698
//     v700 = sshr v687, v696  ; v696 = 13
//     v701 = imul v695, v700
//     v702 = iadd v690, v701
//     v703 = iconst.i32 7
//     v704 = imul v695, v703  ; v703 = 7
//     v705 = isub v693, v704
//     v706 = icmp slt v705, v535  ; v535 = 0
//     v707 = select v706, v536, v537  ; v536 = -1, v537 = 1
//     v708 = iconst.i32 14
//     v709 = sshr v702, v708  ; v708 = 14
//     v710 = imul v707, v709
//     v711 = isub v699, v710
//     v712 = sshr v699, v708  ; v708 = 14
//     v713 = imul v707, v712
//     v714 = iadd v702, v713
//     v715 = iconst.i32 3
//     v716 = imul v707, v715  ; v715 = 3
//     v717 = isub v705, v716
//     v718 = icmp slt v717, v535  ; v535 = 0
//     v719 = select v718, v536, v537  ; v536 = -1, v537 = 1
//     v720 = iconst.i32 15
//     v721 = sshr v714, v720  ; v720 = 15
//     v722 = imul v719, v721
//     v723 = isub v711, v722
//     v724 = sshr v711, v720  ; v720 = 15
//     v725 = imul v719, v724
//     v726 = iadd v714, v725
//     v727 = iconst.i32 1
//     v728 = imul v719, v727  ; v727 = 1
//     v729 = isub v717, v728
//     v730 = iconst.i32 0x0001_a592
//     v731 = sextend.i64 v726
//     v732 = sextend.i64 v730  ; v730 = 0x0001_a592
//     v733 = imul v731, v732
//     v734 = iconst.i64 16
//     v735 = sshr v733, v734  ; v734 = 16
//     v736 = ireduce.i32 v735
//     v737 = sextend.i64 v723
//     v738 = imul v737, v732
//     v739 = sshr v738, v734  ; v734 = 16
//     v740 = ireduce.i32 v739
//     v741 = iconst.i32 0
//     v742 = iconst.i32 1
//     v743 = iconst.i32 2
//     v744 = iconst.i32 3
//     v745 = icmp eq v532, v741  ; v741 = 0
//     v746 = icmp eq v532, v742  ; v742 = 1
//     v747 = icmp eq v532, v743  ; v743 = 2
//     v748 = icmp eq v532, v744  ; v744 = 3
//     v749 = ineg v736
//     v750 = select v747, v749, v736
//     v751 = select v748, v749, v750
//     v752 = select v746, v736, v751
//     v753 = select v745, v736, v752
//     store notrap aligned v259, v0
//     store notrap aligned v506, v0+4
//     store notrap aligned v753, v0+8
//     return
//
// block1:
//     v754 = iconst.i32 0
//     store notrap aligned v754, v0  ; v754 = 0
//     v755 = iconst.i32 0
//     store notrap aligned v755, v0+4  ; v755 = 0
//     v756 = iconst.i32 0
//     store notrap aligned v756, v0+8  ; v756 = 0
//     return
// }
// run: ≈ vec3(0, 1, -0.00000008742278) (tolerance: 0.001)
