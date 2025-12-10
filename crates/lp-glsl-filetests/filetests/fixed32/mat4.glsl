// test riscv32.fixed32
// test run

mat4 main() {
    return mat4(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0) + mat4(0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6);
}

// Generated CLIF
// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = f32const 0x1.400000p2
//     v6 = f32const 0x1.800000p2
//     v7 = f32const 0x1.c00000p2
//     v8 = f32const 0x1.000000p3
//     v9 = f32const 0x1.200000p3
//     v10 = f32const 0x1.400000p3
//     v11 = f32const 0x1.600000p3
//     v12 = f32const 0x1.800000p3
//     v13 = f32const 0x1.a00000p3
//     v14 = f32const 0x1.c00000p3
//     v15 = f32const 0x1.e00000p3
//     v16 = f32const 0x1.000000p4
//     v17 = f32const 0x1.99999ap-4
//     v18 = f32const 0x1.99999ap-3
//     v19 = f32const 0x1.333334p-2
//     v20 = f32const 0x1.99999ap-2
//     v21 = f32const 0x1.000000p-1
//     v22 = f32const 0x1.333334p-1
//     v23 = f32const 0x1.666666p-1
//     v24 = f32const 0x1.99999ap-1
//     v25 = f32const 0x1.ccccccp-1
//     v26 = f32const 0x1.000000p0
//     v27 = f32const 0x1.19999ap0
//     v28 = f32const 0x1.333334p0
//     v29 = f32const 0x1.4cccccp0
//     v30 = f32const 0x1.666666p0
//     v31 = f32const 0x1.800000p0
//     v32 = f32const 0x1.99999ap0
//     v33 = fadd v1, v17  ; v1 = 0x1.000000p0, v17 = 0x1.99999ap-4
//     v34 = fadd v2, v18  ; v2 = 0x1.000000p1, v18 = 0x1.99999ap-3
//     v35 = fadd v3, v19  ; v3 = 0x1.800000p1, v19 = 0x1.333334p-2
//     v36 = fadd v4, v20  ; v4 = 0x1.000000p2, v20 = 0x1.99999ap-2
//     v37 = fadd v5, v21  ; v5 = 0x1.400000p2, v21 = 0x1.000000p-1
//     v38 = fadd v6, v22  ; v6 = 0x1.800000p2, v22 = 0x1.333334p-1
//     v39 = fadd v7, v23  ; v7 = 0x1.c00000p2, v23 = 0x1.666666p-1
//     v40 = fadd v8, v24  ; v8 = 0x1.000000p3, v24 = 0x1.99999ap-1
//     v41 = fadd v9, v25  ; v9 = 0x1.200000p3, v25 = 0x1.ccccccp-1
//     v42 = fadd v10, v26  ; v10 = 0x1.400000p3, v26 = 0x1.000000p0
//     v43 = fadd v11, v27  ; v11 = 0x1.600000p3, v27 = 0x1.19999ap0
//     v44 = fadd v12, v28  ; v12 = 0x1.800000p3, v28 = 0x1.333334p0
//     v45 = fadd v13, v29  ; v13 = 0x1.a00000p3, v29 = 0x1.4cccccp0
//     v46 = fadd v14, v30  ; v14 = 0x1.c00000p3, v30 = 0x1.666666p0
//     v47 = fadd v15, v31  ; v15 = 0x1.e00000p3, v31 = 0x1.800000p0
//     v48 = fadd v16, v32  ; v16 = 0x1.000000p4, v32 = 0x1.99999ap0
//     store notrap aligned v33, v0
//     store notrap aligned v34, v0+4
//     store notrap aligned v35, v0+8
//     store notrap aligned v36, v0+12
//     store notrap aligned v37, v0+16
//     store notrap aligned v38, v0+20
//     store notrap aligned v39, v0+24
//     store notrap aligned v40, v0+28
//     store notrap aligned v41, v0+32
//     store notrap aligned v42, v0+36
//     store notrap aligned v43, v0+40
//     store notrap aligned v44, v0+44
//     store notrap aligned v45, v0+48
//     store notrap aligned v46, v0+52
//     store notrap aligned v47, v0+56
//     store notrap aligned v48, v0+60
//     return
//
// block1:
//     v49 = f32const 0.0
//     store notrap aligned v49, v0  ; v49 = 0.0
//     v50 = f32const 0.0
//     store notrap aligned v50, v0+4  ; v50 = 0.0
//     v51 = f32const 0.0
//     store notrap aligned v51, v0+8  ; v51 = 0.0
//     v52 = f32const 0.0
//     store notrap aligned v52, v0+12  ; v52 = 0.0
//     v53 = f32const 0.0
//     store notrap aligned v53, v0+16  ; v53 = 0.0
//     v54 = f32const 0.0
//     store notrap aligned v54, v0+20  ; v54 = 0.0
//     v55 = f32const 0.0
//     store notrap aligned v55, v0+24  ; v55 = 0.0
//     v56 = f32const 0.0
//     store notrap aligned v56, v0+28  ; v56 = 0.0
//     v57 = f32const 0.0
//     store notrap aligned v57, v0+32  ; v57 = 0.0
//     v58 = f32const 0.0
//     store notrap aligned v58, v0+36  ; v58 = 0.0
//     v59 = f32const 0.0
//     store notrap aligned v59, v0+40  ; v59 = 0.0
//     v60 = f32const 0.0
//     store notrap aligned v60, v0+44  ; v60 = 0.0
//     v61 = f32const 0.0
//     store notrap aligned v61, v0+48  ; v61 = 0.0
//     v62 = f32const 0.0
//     store notrap aligned v62, v0+52  ; v62 = 0.0
//     v63 = f32const 0.0
//     store notrap aligned v63, v0+56  ; v63 = 0.0
//     v64 = f32const 0.0
//     store notrap aligned v64, v0+60  ; v64 = 0.0
//     return
// }
//
// Transformed CLIF
// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = iconst.i32 0x0005_0000
//     v6 = iconst.i32 0x0006_0000
//     v7 = iconst.i32 0x0007_0000
//     v8 = iconst.i32 0x0008_0000
//     v9 = iconst.i32 0x0009_0000
//     v10 = iconst.i32 0x000a_0000
//     v11 = iconst.i32 0x000b_0000
//     v12 = iconst.i32 0x000c_0000
//     v13 = iconst.i32 0x000d_0000
//     v14 = iconst.i32 0x000e_0000
//     v15 = iconst.i32 0x000f_0000
//     v16 = iconst.i32 0x0010_0000
//     v17 = iconst.i32 6554
//     v18 = iconst.i32 0x3333
//     v19 = iconst.i32 0x4ccd
//     v20 = iconst.i32 0x6666
//     v21 = iconst.i32 0x8000
//     v22 = iconst.i32 0x999a
//     v23 = iconst.i32 0xb333
//     v24 = iconst.i32 0xcccd
//     v25 = iconst.i32 0xe666
//     v26 = iconst.i32 0x0001_0000
//     v27 = iconst.i32 0x0001_199a
//     v28 = iconst.i32 0x0001_3333
//     v29 = iconst.i32 0x0001_4ccd
//     v30 = iconst.i32 0x0001_6666
//     v31 = iconst.i32 0x0001_8000
//     v32 = iconst.i32 0x0001_999a
//     v33 = iadd v1, v17  ; v1 = 0x0001_0000, v17 = 6554
//     v34 = iadd v2, v18  ; v2 = 0x0002_0000, v18 = 0x3333
//     v35 = iadd v3, v19  ; v3 = 0x0003_0000, v19 = 0x4ccd
//     v36 = iadd v4, v20  ; v4 = 0x0004_0000, v20 = 0x6666
//     v37 = iadd v5, v21  ; v5 = 0x0005_0000, v21 = 0x8000
//     v38 = iadd v6, v22  ; v6 = 0x0006_0000, v22 = 0x999a
//     v39 = iadd v7, v23  ; v7 = 0x0007_0000, v23 = 0xb333
//     v40 = iadd v8, v24  ; v8 = 0x0008_0000, v24 = 0xcccd
//     v41 = iadd v9, v25  ; v9 = 0x0009_0000, v25 = 0xe666
//     v42 = iadd v10, v26  ; v10 = 0x000a_0000, v26 = 0x0001_0000
//     v43 = iadd v11, v27  ; v11 = 0x000b_0000, v27 = 0x0001_199a
//     v44 = iadd v12, v28  ; v12 = 0x000c_0000, v28 = 0x0001_3333
//     v45 = iadd v13, v29  ; v13 = 0x000d_0000, v29 = 0x0001_4ccd
//     v46 = iadd v14, v30  ; v14 = 0x000e_0000, v30 = 0x0001_6666
//     v47 = iadd v15, v31  ; v15 = 0x000f_0000, v31 = 0x0001_8000
//     v48 = iadd v16, v32  ; v16 = 0x0010_0000, v32 = 0x0001_999a
//     store notrap aligned v33, v0
//     store notrap aligned v34, v0+4
//     store notrap aligned v35, v0+8
//     store notrap aligned v36, v0+12
//     store notrap aligned v37, v0+16
//     store notrap aligned v38, v0+20
//     store notrap aligned v39, v0+24
//     store notrap aligned v40, v0+28
//     store notrap aligned v41, v0+32
//     store notrap aligned v42, v0+36
//     store notrap aligned v43, v0+40
//     store notrap aligned v44, v0+44
//     store notrap aligned v45, v0+48
//     store notrap aligned v46, v0+52
//     store notrap aligned v47, v0+56
//     store notrap aligned v48, v0+60
//     return
//
// block1:
//     v49 = iconst.i32 0
//     store notrap aligned v49, v0  ; v49 = 0
//     v50 = iconst.i32 0
//     store notrap aligned v50, v0+4  ; v50 = 0
//     v51 = iconst.i32 0
//     store notrap aligned v51, v0+8  ; v51 = 0
//     v52 = iconst.i32 0
//     store notrap aligned v52, v0+12  ; v52 = 0
//     v53 = iconst.i32 0
//     store notrap aligned v53, v0+16  ; v53 = 0
//     v54 = iconst.i32 0
//     store notrap aligned v54, v0+20  ; v54 = 0
//     v55 = iconst.i32 0
//     store notrap aligned v55, v0+24  ; v55 = 0
//     v56 = iconst.i32 0
//     store notrap aligned v56, v0+28  ; v56 = 0
//     v57 = iconst.i32 0
//     store notrap aligned v57, v0+32  ; v57 = 0
//     v58 = iconst.i32 0
//     store notrap aligned v58, v0+36  ; v58 = 0
//     v59 = iconst.i32 0
//     store notrap aligned v59, v0+40  ; v59 = 0
//     v60 = iconst.i32 0
//     store notrap aligned v60, v0+44  ; v60 = 0
//     v61 = iconst.i32 0
//     store notrap aligned v61, v0+48  ; v61 = 0
//     v62 = iconst.i32 0
//     store notrap aligned v62, v0+52  ; v62 = 0
//     v63 = iconst.i32 0
//     store notrap aligned v63, v0+56  ; v63 = 0
//     v64 = iconst.i32 0
//     store notrap aligned v64, v0+60  ; v64 = 0
//     return
// }
// run: ≈ mat4(1.1, 2.2, 3.3, 4.4, 5.5, 6.6, 7.7, 8.8, 9.9, 11.0, 12.1, 13.2, 14.3, 15.4, 16.5, 17.6)
