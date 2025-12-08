// test compile
// test run

float main() {
    return cos(0.0);
}

// function u0:0() -> f32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0.0
//     v1 = call fn0(v0)  ; v0 = 0.0
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 1 (tolerance: 0.001)
