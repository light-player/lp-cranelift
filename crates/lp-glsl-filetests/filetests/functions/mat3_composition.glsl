// test compile

// target riscv32.fixed32
mat3 scale(mat3 m, float s) {
    return m * s;
}

mat3 main() {
    mat3 m = mat3(1.0);
    return scale(m, 2.0);
}

// function u0:0(i64 sret) apple_aarch64 {
//     ss0 = explicit_slot 36, align = 4
//     sig0 = (i64 sret, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) apple_aarch64
//     fn0 = colocated u0:0 sig0
//
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0.0
//     v3 = f32const 0x1.000000p1
//     v4 = stack_addr.i64 ss0
//     call fn0(v4, v1, v2, v2, v2, v1, v2, v2, v2, v1, v3)  ; v1 = 0x1.000000p0, v2 = 0.0, v2 = 0.0, v2 = 0.0, v1 = 0x1.000000p0, v2 = 0.0, v2 = 0.0, v2 = 0.0, v1 = 0x1.000000p0, v3 = 0x1.000000p1
//     v5 = load.f32 notrap aligned v4
//     v6 = load.f32 notrap aligned v4+4
//     v7 = load.f32 notrap aligned v4+8
//     v8 = load.f32 notrap aligned v4+12
//     v9 = load.f32 notrap aligned v4+16
//     v10 = load.f32 notrap aligned v4+20
//     v11 = load.f32 notrap aligned v4+24
//     v12 = load.f32 notrap aligned v4+28
//     v13 = load.f32 notrap aligned v4+32
//     store notrap aligned v5, v0
//     store notrap aligned v6, v0+4
//     store notrap aligned v7, v0+8
//     store notrap aligned v8, v0+12
//     store notrap aligned v9, v0+16
//     store notrap aligned v10, v0+20
//     store notrap aligned v11, v0+24
//     store notrap aligned v12, v0+28
//     store notrap aligned v13, v0+32
//     return
//
// block1:
//     v14 = f32const 0.0
//     store notrap aligned v14, v0  ; v14 = 0.0
//     v15 = f32const 0.0
//     store notrap aligned v15, v0+4  ; v15 = 0.0
//     v16 = f32const 0.0
//     store notrap aligned v16, v0+8  ; v16 = 0.0
//     v17 = f32const 0.0
//     store notrap aligned v17, v0+12  ; v17 = 0.0
//     v18 = f32const 0.0
//     store notrap aligned v18, v0+16  ; v18 = 0.0
//     v19 = f32const 0.0
//     store notrap aligned v19, v0+20  ; v19 = 0.0
//     v20 = f32const 0.0
//     store notrap aligned v20, v0+24  ; v20 = 0.0
//     v21 = f32const 0.0
//     store notrap aligned v21, v0+28  ; v21 = 0.0
//     v22 = f32const 0.0
//     store notrap aligned v22, v0+32  ; v22 = 0.0
//     return
// }

