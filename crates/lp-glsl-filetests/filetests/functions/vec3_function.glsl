// test compile
// test run

vec3 scale(vec3 v, float s) {
    return v * s;
}

vec3 main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return scale(v, 2.0);  // (2.0, 4.0, 6.0)
}

// function u0:0(i64 sret) apple_aarch64 {
//     ss0 = explicit_slot 12, align = 4
//     sig0 = (i64 sret, f32, f32, f32, f32) apple_aarch64
//     fn0 = colocated u0:0 sig0
//
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p1
//     v5 = stack_addr.i64 ss0
//     call fn0(v5, v1, v2, v3, v4)  ; v1 = 0x1.000000p0, v2 = 0x1.000000p1, v3 = 0x1.800000p1, v4 = 0x1.000000p1
//     v6 = load.f32 notrap aligned v5
//     v7 = load.f32 notrap aligned v5+4
//     v8 = load.f32 notrap aligned v5+8
//     store notrap aligned v6, v0
//     store notrap aligned v7, v0+4
//     store notrap aligned v8, v0+8
//     return
//
// block1:
//     v9 = f32const 0.0
//     store notrap aligned v9, v0  ; v9 = 0.0
//     v10 = f32const 0.0
//     store notrap aligned v10, v0+4  ; v10 = 0.0
//     v11 = f32const 0.0
//     store notrap aligned v11, v0+8  ; v11 = 0.0
//     return
// }
// run: ≈ vec3(2.0, 4.0, 6.0) (tolerance: 0.001)
