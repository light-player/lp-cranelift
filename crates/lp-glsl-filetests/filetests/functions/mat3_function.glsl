// test compile

mat3 identity() {
    return mat3(1.0);
}

mat3 main() {
    return identity();
}

// function u0:0(i64 sret) apple_aarch64 {
//     ss0 = explicit_slot 36, align = 4
//     sig0 = (i64 sret) apple_aarch64
//     fn0 = colocated u0:0 sig0
//
// block0(v0: i64):
//     v1 = stack_addr.i64 ss0
//     call fn0(v1)
//     v2 = load.f32 notrap aligned v1
//     v3 = load.f32 notrap aligned v1+4
//     v4 = load.f32 notrap aligned v1+8
//     v5 = load.f32 notrap aligned v1+12
//     v6 = load.f32 notrap aligned v1+16
//     v7 = load.f32 notrap aligned v1+20
//     v8 = load.f32 notrap aligned v1+24
//     v9 = load.f32 notrap aligned v1+28
//     v10 = load.f32 notrap aligned v1+32
//     store notrap aligned v2, v0
//     store notrap aligned v3, v0+4
//     store notrap aligned v4, v0+8
//     store notrap aligned v5, v0+12
//     store notrap aligned v6, v0+16
//     store notrap aligned v7, v0+20
//     store notrap aligned v8, v0+24
//     store notrap aligned v9, v0+28
//     store notrap aligned v10, v0+32
//     return
//
// block1:
//     v11 = f32const 0.0
//     store notrap aligned v11, v0  ; v11 = 0.0
//     v12 = f32const 0.0
//     store notrap aligned v12, v0+4  ; v12 = 0.0
//     v13 = f32const 0.0
//     store notrap aligned v13, v0+8  ; v13 = 0.0
//     v14 = f32const 0.0
//     store notrap aligned v14, v0+12  ; v14 = 0.0
//     v15 = f32const 0.0
//     store notrap aligned v15, v0+16  ; v15 = 0.0
//     v16 = f32const 0.0
//     store notrap aligned v16, v0+20  ; v16 = 0.0
//     v17 = f32const 0.0
//     store notrap aligned v17, v0+24  ; v17 = 0.0
//     v18 = f32const 0.0
//     store notrap aligned v18, v0+28  ; v18 = 0.0
//     v19 = f32const 0.0
//     store notrap aligned v19, v0+32  ; v19 = 0.0
//     return
// }
