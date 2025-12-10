// test compile
// test run
// target riscv32.fixed32

mat3 main() {
    vec3 u = vec3(1.0, 2.0, 3.0);
    vec3 v = vec3(4.0, 5.0, 6.0);
    return outerProduct(u, v);
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v25 = iconst.i32 0x0001_0000
//     v26 = iconst.i32 0x0002_0000
//     v27 = iconst.i32 0x0003_0000
//     v28 = iconst.i32 0x0004_0000
//     v29 = iconst.i32 0x0005_0000
//     v30 = iconst.i32 0x0006_0000
//     v31 = sextend.i64 v25  ; v25 = 0x0001_0000
//     v32 = sextend.i64 v28  ; v28 = 0x0004_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = sextend.i64 v26  ; v26 = 0x0002_0000
//     v38 = sextend.i64 v28  ; v28 = 0x0004_0000
//     v39 = imul v37, v38
//     v40 = iconst.i64 16
//     v41 = sshr v39, v40  ; v40 = 16
//     v42 = ireduce.i32 v41
//     v43 = sextend.i64 v27  ; v27 = 0x0003_0000
//     v44 = sextend.i64 v28  ; v28 = 0x0004_0000
//     v45 = imul v43, v44
//     v46 = iconst.i64 16
//     v47 = sshr v45, v46  ; v46 = 16
//     v48 = ireduce.i32 v47
//     v49 = sextend.i64 v25  ; v25 = 0x0001_0000
//     v50 = sextend.i64 v29  ; v29 = 0x0005_0000
//     v51 = imul v49, v50
//     v52 = iconst.i64 16
//     v53 = sshr v51, v52  ; v52 = 16
//     v54 = ireduce.i32 v53
//     v55 = sextend.i64 v26  ; v26 = 0x0002_0000
//     v56 = sextend.i64 v29  ; v29 = 0x0005_0000
//     v57 = imul v55, v56
//     v58 = iconst.i64 16
//     v59 = sshr v57, v58  ; v58 = 16
//     v60 = ireduce.i32 v59
//     v61 = sextend.i64 v27  ; v27 = 0x0003_0000
//     v62 = sextend.i64 v29  ; v29 = 0x0005_0000
//     v63 = imul v61, v62
//     v64 = iconst.i64 16
//     v65 = sshr v63, v64  ; v64 = 16
//     v66 = ireduce.i32 v65
//     v67 = sextend.i64 v25  ; v25 = 0x0001_0000
//     v68 = sextend.i64 v30  ; v30 = 0x0006_0000
//     v69 = imul v67, v68
//     v70 = iconst.i64 16
//     v71 = sshr v69, v70  ; v70 = 16
//     v72 = ireduce.i32 v71
//     v73 = sextend.i64 v26  ; v26 = 0x0002_0000
//     v74 = sextend.i64 v30  ; v30 = 0x0006_0000
//     v75 = imul v73, v74
//     v76 = iconst.i64 16
//     v77 = sshr v75, v76  ; v76 = 16
//     v78 = ireduce.i32 v77
//     v79 = sextend.i64 v27  ; v27 = 0x0003_0000
//     v80 = sextend.i64 v30  ; v30 = 0x0006_0000
//     v81 = imul v79, v80
//     v82 = iconst.i64 16
//     v83 = sshr v81, v82  ; v82 = 16
//     v84 = ireduce.i32 v83
//     store notrap aligned v36, v0
//     store notrap aligned v42, v0+4
//     store notrap aligned v48, v0+8
//     store notrap aligned v54, v0+12
//     store notrap aligned v60, v0+16
//     store notrap aligned v66, v0+20
//     store notrap aligned v72, v0+24
//     store notrap aligned v78, v0+28
//     store notrap aligned v84, v0+32
//     return
//
// block1:
//     v85 = iconst.i32 0
//     store notrap aligned v85, v0  ; v85 = 0
//     v86 = iconst.i32 0
//     store notrap aligned v86, v0+4  ; v86 = 0
//     v87 = iconst.i32 0
//     store notrap aligned v87, v0+8  ; v87 = 0
//     v88 = iconst.i32 0
//     store notrap aligned v88, v0+12  ; v88 = 0
//     v89 = iconst.i32 0
//     store notrap aligned v89, v0+16  ; v89 = 0
//     v90 = iconst.i32 0
//     store notrap aligned v90, v0+20  ; v90 = 0
//     v91 = iconst.i32 0
//     store notrap aligned v91, v0+24  ; v91 = 0
//     v92 = iconst.i32 0
//     store notrap aligned v92, v0+28  ; v92 = 0
//     v93 = iconst.i32 0
//     store notrap aligned v93, v0+32  ; v93 = 0
//     return
// }
// run: ≈ mat3(4, 8, 12, 5, 10, 15, 6, 12, 18) (tolerance: 0.01)
