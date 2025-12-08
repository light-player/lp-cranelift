// test compile
// test run

vec2 main() {
    return floor(vec2(3.7, -2.3));  // Should return (3.0, -3.0)
}

// function u0:0() -> f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.d9999ap1
//     v1 = f32const 0x1.266666p1
//     v2 = fneg v1  ; v1 = 0x1.266666p1
//     v3 = floor v0  ; v0 = 0x1.d9999ap1
//     v4 = floor v2
//     return v3, v4
//
// block1:
//     v5 = f32const 0.0
//     v6 = f32const 0.0
//     return v5, v6  ; v5 = 0.0, v6 = 0.0
// }
// run: ≈ vec2(3.0, -3.0) (tolerance: 0.01)
