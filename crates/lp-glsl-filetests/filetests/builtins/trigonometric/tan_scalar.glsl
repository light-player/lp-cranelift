// test compile
// test run

float main() {
    float pi_4 = 0.785398163;  // π/4
    return tan(pi_4);
}

// function u0:0() -> f32 apple_aarch64 {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.921fb6p-1
//     v1 = call fn0(v0)  ; v0 = 0x1.921fb6p-1
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 1 (tolerance: 0.001)
