// test compile

// target riscv32.fixed32
mat2 identity() {
    return mat2(1.0);
}

mat2 main() {
    return identity();
}

// function u0:0(i32 sret) system_v {
//     ss0 = explicit_slot 16, align = 4
//     sig0 = (i32 sret) system_v
//     fn0 = colocated u0:0 sig0
//
// block0(v0: i32):
//     v1 = stack_addr.i32 ss0
//     call fn0(v1)
//     v10 = load.i32 notrap aligned v1
//     v11 = load.i32 notrap aligned v1+4
//     v12 = load.i32 notrap aligned v1+8
//     v13 = load.i32 notrap aligned v1+12
//     store notrap aligned v10, v0
//     store notrap aligned v11, v0+4
//     store notrap aligned v12, v0+8
//     store notrap aligned v13, v0+12
//     return
//
// block1:
//     v14 = iconst.i32 0
//     store notrap aligned v14, v0  ; v14 = 0
//     v15 = iconst.i32 0
//     store notrap aligned v15, v0+4  ; v15 = 0
//     v16 = iconst.i32 0
//     store notrap aligned v16, v0+8  ; v16 = 0
//     v17 = iconst.i32 0
//     store notrap aligned v17, v0+12  ; v17 = 0
//     return
// }

