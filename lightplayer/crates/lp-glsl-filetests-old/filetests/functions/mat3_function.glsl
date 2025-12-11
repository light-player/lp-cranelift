// test compile

// target riscv32.fixed32
mat3 identity() {
    return mat3(1.0);
}

mat3 main() {
    return identity();
}

// function u0:0(i32 sret) system_v {
//     ss0 = explicit_slot 36, align = 4
//     sig0 = (i32 sret) system_v
//     fn0 = colocated u0:0 sig0
//
// block0(v0: i32):
//     v1 = stack_addr.i32 ss0
//     call fn0(v1)
//     v20 = load.i32 notrap aligned v1
//     v21 = load.i32 notrap aligned v1+4
//     v22 = load.i32 notrap aligned v1+8
//     v23 = load.i32 notrap aligned v1+12
//     v24 = load.i32 notrap aligned v1+16
//     v25 = load.i32 notrap aligned v1+20
//     v26 = load.i32 notrap aligned v1+24
//     v27 = load.i32 notrap aligned v1+28
//     v28 = load.i32 notrap aligned v1+32
//     store notrap aligned v20, v0
//     store notrap aligned v21, v0+4
//     store notrap aligned v22, v0+8
//     store notrap aligned v23, v0+12
//     store notrap aligned v24, v0+16
//     store notrap aligned v25, v0+20
//     store notrap aligned v26, v0+24
//     store notrap aligned v27, v0+28
//     store notrap aligned v28, v0+32
//     return
//
// block1:
//     v29 = iconst.i32 0
//     store notrap aligned v29, v0  ; v29 = 0
//     v30 = iconst.i32 0
//     store notrap aligned v30, v0+4  ; v30 = 0
//     v31 = iconst.i32 0
//     store notrap aligned v31, v0+8  ; v31 = 0
//     v32 = iconst.i32 0
//     store notrap aligned v32, v0+12  ; v32 = 0
//     v33 = iconst.i32 0
//     store notrap aligned v33, v0+16  ; v33 = 0
//     v34 = iconst.i32 0
//     store notrap aligned v34, v0+20  ; v34 = 0
//     v35 = iconst.i32 0
//     store notrap aligned v35, v0+24  ; v35 = 0
//     v36 = iconst.i32 0
//     store notrap aligned v36, v0+28  ; v36 = 0
//     v37 = iconst.i32 0
//     store notrap aligned v37, v0+32  ; v37 = 0
//     return
// }



