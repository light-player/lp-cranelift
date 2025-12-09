// test compile

// target riscv32
mat3 transpose_mat(mat3 m) {
    return transpose(m);
}

mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose_mat(m);
}

// function u0:0(i32 sret) system_v {
//     ss0 = explicit_slot 36, align = 4
//     sig0 = (i32 sret, f32, f32, f32, f32, f32, f32, f32, f32, f32) system_v
//     fn0 = colocated u0:0 sig0
//
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
//     v10 = stack_addr.i32 ss0
//     call fn0(v10, v1, v2, v3, v4, v5, v6, v7, v8, v9)  ; v1 = 0x1.000000p0, v2 = 0x1.000000p1, v3 = 0x1.800000p1, v4 = 0x1.000000p2, v5 = 0x1.400000p2, v6 = 0x1.800000p2, v7 = 0x1.c00000p2, v8 = 0x1.000000p3, v9 = 0x1.200000p3
//     v11 = load.f32 notrap aligned v10
//     v12 = load.f32 notrap aligned v10+4
//     v13 = load.f32 notrap aligned v10+8
//     v14 = load.f32 notrap aligned v10+12
//     v15 = load.f32 notrap aligned v10+16
//     v16 = load.f32 notrap aligned v10+20
//     v17 = load.f32 notrap aligned v10+24
//     v18 = load.f32 notrap aligned v10+28
//     v19 = load.f32 notrap aligned v10+32
//     store notrap aligned v11, v0
//     store notrap aligned v12, v0+4
//     store notrap aligned v13, v0+8
//     store notrap aligned v14, v0+12
//     store notrap aligned v15, v0+16
//     store notrap aligned v16, v0+20
//     store notrap aligned v17, v0+24
//     store notrap aligned v18, v0+28
//     store notrap aligned v19, v0+32
//     return
//
// block1:
//     v20 = f32const 0.0
//     store notrap aligned v20, v0  ; v20 = 0.0
//     v21 = f32const 0.0
//     store notrap aligned v21, v0+4  ; v21 = 0.0
//     v22 = f32const 0.0
//     store notrap aligned v22, v0+8  ; v22 = 0.0
//     v23 = f32const 0.0
//     store notrap aligned v23, v0+12  ; v23 = 0.0
//     v24 = f32const 0.0
//     store notrap aligned v24, v0+16  ; v24 = 0.0
//     v25 = f32const 0.0
//     store notrap aligned v25, v0+20  ; v25 = 0.0
//     v26 = f32const 0.0
//     store notrap aligned v26, v0+24  ; v26 = 0.0
//     v27 = f32const 0.0
//     store notrap aligned v27, v0+28  ; v27 = 0.0
//     v28 = f32const 0.0
//     store notrap aligned v28, v0+32  ; v28 = 0.0
//     return
// }
