// test compile

mat4 identity() {
    return mat4(1.0);
}

mat4 main() {
    return identity();
}

// function u0:0(i64 sret) apple_aarch64 {
//     ss0 = explicit_slot 64, align = 4
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
//     v11 = load.f32 notrap aligned v1+36
//     v12 = load.f32 notrap aligned v1+40
//     v13 = load.f32 notrap aligned v1+44
//     v14 = load.f32 notrap aligned v1+48
//     v15 = load.f32 notrap aligned v1+52
//     v16 = load.f32 notrap aligned v1+56
//     v17 = load.f32 notrap aligned v1+60
//     store notrap aligned v2, v0
//     store notrap aligned v3, v0+4
//     store notrap aligned v4, v0+8
//     store notrap aligned v5, v0+12
//     store notrap aligned v6, v0+16
//     store notrap aligned v7, v0+20
//     store notrap aligned v8, v0+24
//     store notrap aligned v9, v0+28
//     store notrap aligned v10, v0+32
//     store notrap aligned v11, v0+36
//     store notrap aligned v12, v0+40
//     store notrap aligned v13, v0+44
//     store notrap aligned v14, v0+48
//     store notrap aligned v15, v0+52
//     store notrap aligned v16, v0+56
//     store notrap aligned v17, v0+60
//     return
//
// block1:
//     v18 = f32const 0.0
//     store notrap aligned v18, v0  ; v18 = 0.0
//     v19 = f32const 0.0
//     store notrap aligned v19, v0+4  ; v19 = 0.0
//     v20 = f32const 0.0
//     store notrap aligned v20, v0+8  ; v20 = 0.0
//     v21 = f32const 0.0
//     store notrap aligned v21, v0+12  ; v21 = 0.0
//     v22 = f32const 0.0
//     store notrap aligned v22, v0+16  ; v22 = 0.0
//     v23 = f32const 0.0
//     store notrap aligned v23, v0+20  ; v23 = 0.0
//     v24 = f32const 0.0
//     store notrap aligned v24, v0+24  ; v24 = 0.0
//     v25 = f32const 0.0
//     store notrap aligned v25, v0+28  ; v25 = 0.0
//     v26 = f32const 0.0
//     store notrap aligned v26, v0+32  ; v26 = 0.0
//     v27 = f32const 0.0
//     store notrap aligned v27, v0+36  ; v27 = 0.0
//     v28 = f32const 0.0
//     store notrap aligned v28, v0+40  ; v28 = 0.0
//     v29 = f32const 0.0
//     store notrap aligned v29, v0+44  ; v29 = 0.0
//     v30 = f32const 0.0
//     store notrap aligned v30, v0+48  ; v30 = 0.0
//     v31 = f32const 0.0
//     store notrap aligned v31, v0+52  ; v31 = 0.0
//     v32 = f32const 0.0
//     store notrap aligned v32, v0+56  ; v32 = 0.0
//     v33 = f32const 0.0
//     store notrap aligned v33, v0+60  ; v33 = 0.0
//     return
// }
