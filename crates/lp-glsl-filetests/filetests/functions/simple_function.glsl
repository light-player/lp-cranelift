// test compile
// test run

float square(float x) {
    return x * x;
}

float main() {
    return square(5.0);  // 25.0
}

// function u0:0() -> f32 apple_aarch64 {
//     sig0 = (f32) -> f32 apple_aarch64
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = call fn0(v0)  ; v0 = 0x1.400000p2
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 25 (tolerance: 0.01)
