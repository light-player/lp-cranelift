// test compile
// test run

float main() {
    return sqrt(16.0);  // 4.0
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p4
//     v1 = sqrt v0  ; v0 = 0x1.000000p4
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 0 (tolerance: 0.01)
