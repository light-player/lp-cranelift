// test compile
// test run

float main() {
    float deg = 180.0;
    float rad = radians(deg);
    return degrees(rad);
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.680000p7
//     v1 = f32const 0x1.1df46ap-6
//     v2 = fmul v0, v1  ; v0 = 0x1.680000p7, v1 = 0x1.1df46ap-6
//     v3 = f32const 0x1.ca5dc2p5
//     v4 = fmul v2, v3  ; v3 = 0x1.ca5dc2p5
//     return v4
//
// block1:
//     v5 = f32const 0.0
//     return v5  ; v5 = 0.0
// }
// run: ~= 180 (tolerance: 0.1)
