// test compile
// test run

float main() {
    return clamp(7.0, 2.0, 5.0);  // 5.0
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.c00000p2
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.400000p2
//     v3 = fmax v0, v1  ; v0 = 0x1.c00000p2, v1 = 0x1.000000p1
//     v4 = fmin v3, v2  ; v2 = 0x1.400000p2
//     return v4
//
// block1:
//     v5 = f32const 0.0
//     return v5  ; v5 = 0.0
// }
// run: ~= 5 (tolerance: 0.01)
