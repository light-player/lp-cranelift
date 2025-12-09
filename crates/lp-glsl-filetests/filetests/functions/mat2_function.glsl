// test compile

// target riscv32
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
//     v2 = load.f32 notrap aligned v1
//     v3 = load.f32 notrap aligned v1+4
//     v4 = load.f32 notrap aligned v1+8
//     v5 = load.f32 notrap aligned v1+12
//     store notrap aligned v2, v0
//     store notrap aligned v3, v0+4
//     store notrap aligned v4, v0+8
//     store notrap aligned v5, v0+12
//     return
//
// block1:
//     v6 = f32const 0.0
//     store notrap aligned v6, v0  ; v6 = 0.0
//     v7 = f32const 0.0
//     store notrap aligned v7, v0+4  ; v7 = 0.0
//     v8 = f32const 0.0
//     store notrap aligned v8, v0+8  ; v8 = 0.0
//     v9 = f32const 0.0
//     store notrap aligned v9, v0+12  ; v9 = 0.0
//     return
// }
