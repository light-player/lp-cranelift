// test compile
// test run

float main() {
    return atan(1.0);
}

// function u0:0() -> f32 apple_aarch64 {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = call fn0(v0)  ; v0 = 0x1.000000p0
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 0.785398 (tolerance: 0.001)  // π/4
