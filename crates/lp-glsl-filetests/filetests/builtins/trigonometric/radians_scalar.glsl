// test compile
// test run

float main() {
    return radians(180.0);  // Should be approximately π
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.680000p7
//     v1 = f32const 0x1.1df46ap-6
//     v2 = fmul v0, v1  ; v0 = 0x1.680000p7, v1 = 0x1.1df46ap-6
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 3.1415927 (tolerance: 0.01)
