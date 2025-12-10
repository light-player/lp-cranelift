// test compile

// target riscv32.fixed32
mat4 identity() {
    return mat4(1.0);
}

mat4 main() {
    return identity();
}

// function u0:0(i32 sret) system_v {
//     ss0 = explicit_slot 64, align = 4
//     sig0 = (i32 sret) system_v
//     fn0 = colocated u0:0 sig0
//
// block0(v0: i32):
//     v1 = stack_addr.i32 ss0
//     call fn0(v1)
//     v34 = load.i32 notrap aligned v1
//     v35 = load.i32 notrap aligned v1+4
//     v36 = load.i32 notrap aligned v1+8
//     v37 = load.i32 notrap aligned v1+12
//     v38 = load.i32 notrap aligned v1+16
//     v39 = load.i32 notrap aligned v1+20
//     v40 = load.i32 notrap aligned v1+24
//     v41 = load.i32 notrap aligned v1+28
//     v42 = load.i32 notrap aligned v1+32
//     v43 = load.i32 notrap aligned v1+36
//     v44 = load.i32 notrap aligned v1+40
//     v45 = load.i32 notrap aligned v1+44
//     v46 = load.i32 notrap aligned v1+48
//     v47 = load.i32 notrap aligned v1+52
//     v48 = load.i32 notrap aligned v1+56
//     v49 = load.i32 notrap aligned v1+60
//     store notrap aligned v34, v0
//     store notrap aligned v35, v0+4
//     store notrap aligned v36, v0+8
//     store notrap aligned v37, v0+12
//     store notrap aligned v38, v0+16
//     store notrap aligned v39, v0+20
//     store notrap aligned v40, v0+24
//     store notrap aligned v41, v0+28
//     store notrap aligned v42, v0+32
//     store notrap aligned v43, v0+36
//     store notrap aligned v44, v0+40
//     store notrap aligned v45, v0+44
//     store notrap aligned v46, v0+48
//     store notrap aligned v47, v0+52
//     store notrap aligned v48, v0+56
//     store notrap aligned v49, v0+60
//     return
//
// block1:
//     v50 = iconst.i32 0
//     store notrap aligned v50, v0  ; v50 = 0
//     v51 = iconst.i32 0
//     store notrap aligned v51, v0+4  ; v51 = 0
//     v52 = iconst.i32 0
//     store notrap aligned v52, v0+8  ; v52 = 0
//     v53 = iconst.i32 0
//     store notrap aligned v53, v0+12  ; v53 = 0
//     v54 = iconst.i32 0
//     store notrap aligned v54, v0+16  ; v54 = 0
//     v55 = iconst.i32 0
//     store notrap aligned v55, v0+20  ; v55 = 0
//     v56 = iconst.i32 0
//     store notrap aligned v56, v0+24  ; v56 = 0
//     v57 = iconst.i32 0
//     store notrap aligned v57, v0+28  ; v57 = 0
//     v58 = iconst.i32 0
//     store notrap aligned v58, v0+32  ; v58 = 0
//     v59 = iconst.i32 0
//     store notrap aligned v59, v0+36  ; v59 = 0
//     v60 = iconst.i32 0
//     store notrap aligned v60, v0+40  ; v60 = 0
//     v61 = iconst.i32 0
//     store notrap aligned v61, v0+44  ; v61 = 0
//     v62 = iconst.i32 0
//     store notrap aligned v62, v0+48  ; v62 = 0
//     v63 = iconst.i32 0
//     store notrap aligned v63, v0+52  ; v63 = 0
//     v64 = iconst.i32 0
//     store notrap aligned v64, v0+56  ; v64 = 0
//     v65 = iconst.i32 0
//     store notrap aligned v65, v0+60  ; v65 = 0
//     return
// }
