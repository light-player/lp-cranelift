// test compile
// test run

float main() {
    return floor(3.7);  // Should return 3.0
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.d9999ap1
//     v1 = floor v0  ; v0 = 0x1.d9999ap1
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 3.0 (tolerance: 0.01)
