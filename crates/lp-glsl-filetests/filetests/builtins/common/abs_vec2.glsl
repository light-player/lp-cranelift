// test compile
// test run

vec2 main() {
    return abs(vec2(-1.5, 2.3));  // Should return (1.5, 2.3)
}

// function u0:0() -> f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.800000p0
//     v1 = fneg v0  ; v0 = 0x1.800000p0
//     v2 = f32const 0x1.266666p1
//     v3 = fabs v1
//     v4 = fabs v2  ; v2 = 0x1.266666p1
//     return v3, v4
//
// block1:
//     v5 = f32const 0.0
//     v6 = f32const 0.0
//     return v5, v6  ; v5 = 0.0, v6 = 0.0
// }
// run: ≈ vec2(1.5, 2.3) (tolerance: 0.01)
