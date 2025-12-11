// test compile
// test run
// target riscv32.fixed32

mat3 main() {
    vec3 u = vec3(1.0, 2.0, 3.0);
    vec3 v = vec3(4.0, 5.0, 6.0);
    return outerProduct(u, v);
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = iconst.i32 0x0005_0000
//     v6 = iconst.i32 0x0006_0000
//     v7 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v8 = sextend.i64 v4  ; v4 = 0x0004_0000
//     v9 = imul v7, v8
//     v10 = iconst.i64 16
//     v11 = sshr v9, v10  ; v10 = 16
//     v12 = ireduce.i32 v11
//     v13 = sextend.i64 v2  ; v2 = 0x0002_0000
//     v14 = sextend.i64 v4  ; v4 = 0x0004_0000
//     v15 = imul v13, v14
//     v16 = iconst.i64 16
//     v17 = sshr v15, v16  ; v16 = 16
//     v18 = ireduce.i32 v17
//     v19 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v20 = sextend.i64 v4  ; v4 = 0x0004_0000
//     v21 = imul v19, v20
//     v22 = iconst.i64 16
//     v23 = sshr v21, v22  ; v22 = 16
//     v24 = ireduce.i32 v23
//     v25 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v26 = sextend.i64 v5  ; v5 = 0x0005_0000
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = sextend.i64 v2  ; v2 = 0x0002_0000
//     v32 = sextend.i64 v5  ; v5 = 0x0005_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v38 = sextend.i64 v5  ; v5 = 0x0005_0000
//     v39 = imul v37, v38
//     v40 = iconst.i64 16
//     v41 = sshr v39, v40  ; v40 = 16
//     v42 = ireduce.i32 v41
//     v43 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v44 = sextend.i64 v6  ; v6 = 0x0006_0000
//     v45 = imul v43, v44
//     v46 = iconst.i64 16
//     v47 = sshr v45, v46  ; v46 = 16
//     v48 = ireduce.i32 v47
//     v49 = sextend.i64 v2  ; v2 = 0x0002_0000
//     v50 = sextend.i64 v6  ; v6 = 0x0006_0000
//     v51 = imul v49, v50
//     v52 = iconst.i64 16
//     v53 = sshr v51, v52  ; v52 = 16
//     v54 = ireduce.i32 v53
//     v55 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v56 = sextend.i64 v6  ; v6 = 0x0006_0000
//     v57 = imul v55, v56
//     v58 = iconst.i64 16
//     v59 = sshr v57, v58  ; v58 = 16
//     v60 = ireduce.i32 v59
//     store notrap aligned v12, v0
//     store notrap aligned v18, v0+4
//     store notrap aligned v24, v0+8
//     store notrap aligned v30, v0+12
//     store notrap aligned v36, v0+16
//     store notrap aligned v42, v0+20
//     store notrap aligned v48, v0+24
//     store notrap aligned v54, v0+28
//     store notrap aligned v60, v0+32
//     return
//
// block1:
//     v61 = iconst.i32 0
//     store notrap aligned v61, v0  ; v61 = 0
//     v62 = iconst.i32 0
//     store notrap aligned v62, v0+4  ; v62 = 0
//     v63 = iconst.i32 0
//     store notrap aligned v63, v0+8  ; v63 = 0
//     v64 = iconst.i32 0
//     store notrap aligned v64, v0+12  ; v64 = 0
//     v65 = iconst.i32 0
//     store notrap aligned v65, v0+16  ; v65 = 0
//     v66 = iconst.i32 0
//     store notrap aligned v66, v0+20  ; v66 = 0
//     v67 = iconst.i32 0
//     store notrap aligned v67, v0+24  ; v67 = 0
//     v68 = iconst.i32 0
//     store notrap aligned v68, v0+28  ; v68 = 0
//     v69 = iconst.i32 0
//     store notrap aligned v69, v0+32  ; v69 = 0
//     return
// }
// run: ≈ mat3(4, 8, 12, 5, 10, 15, 6, 12, 18) (tolerance: 0.01)
