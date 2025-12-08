// test compile
// test run

float main() {
    return min(5.0, 3.0);  // 3.0
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = f32const 0x1.800000p1
//     v2 = fmin v0, v1  ; v0 = 0x1.400000p2, v1 = 0x1.800000p1
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 3 (tolerance: 0.01)
