// test compile
// test run

vec3 main() {
    vec3 angles = vec3(0.0, 0.785398163, 1.570796327); // 0, π/4, π/2
    return tan(angles);
}

// function u0:0() -> f32, f32, f32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0x1.921fb6p-1
//     v2 = f32const 0x1.921fb6p0
//     v3 = call fn0(v0)  ; v0 = 0.0
//     v4 = call fn0(v1)  ; v1 = 0x1.921fb6p-1
//     v5 = call fn0(v2)  ; v2 = 0x1.921fb6p0
//     return v3, v4, v5
//
// block1:
//     v6 = f32const 0.0
//     v7 = f32const 0.0
//     v8 = f32const 0.0
//     return v6, v7, v8  ; v6 = 0.0, v7 = 0.0, v8 = 0.0
// }
// run: ≈ vec3(0.00000000000000000000000000000000000000000001, 0.000000000000000000000000000000000011613976, 0.000000000000000000000000000000000000000000001) (tolerance: 0.1)
