// test compile
// test run

vec3 main() {
    vec3 angles = vec3(0.0, 1.570796327, 3.141592654); // 0, π/2, π
    return sin(angles);
}

// function u0:0(i64 sret) apple_aarch64 {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0(v0: i64):
//     v1 = f32const 0.0
//     v2 = f32const 0x1.921fb6p0
//     v3 = f32const 0x1.921fb6p1
//     v4 = call fn0(v1)  ; v1 = 0.0
//     v5 = call fn0(v2)  ; v2 = 0x1.921fb6p0
//     v6 = call fn0(v3)  ; v3 = 0x1.921fb6p1
//     store notrap aligned v4, v0
//     store notrap aligned v5, v0+4
//     store notrap aligned v6, v0+8
//     return
//
// block1:
//     v7 = f32const 0.0
//     store notrap aligned v7, v0  ; v7 = 0.0
//     v8 = f32const 0.0
//     store notrap aligned v8, v0+4  ; v8 = 0.0
//     v9 = f32const 0.0
//     store notrap aligned v9, v0+8  ; v9 = 0.0
//     return
// }
// run: ≈ vec3(0, 1, -0.00000008742278) (tolerance: 0.001)
