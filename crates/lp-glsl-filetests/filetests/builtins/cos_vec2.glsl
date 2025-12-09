// test compile
// test run
// target riscv32

vec2 main() {
    vec2 angles = vec2(0.0, 3.141592654); // 0, π
    return cos(angles);
}

// function u0:0() -> f32, f32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0x1.921fb6p1
//     v2 = call fn0(v0)  ; v0 = 0.0
//     v3 = call fn0(v1)  ; v1 = 0x1.921fb6p1
//     return v2, v3
//
// block1:
//     v4 = f32const 0.0
//     v5 = f32const 0.0
//     return v4, v5  ; v4 = 0.0, v5 = 0.0
// }
// run: ≈ vec2(1.0, -1.0) (tolerance: 0.001)
