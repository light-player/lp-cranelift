// test compile
// test run

float main() {
    vec2 v = vec2(3.0, 4.0);
    return length(v);  // sqrt(9 + 16) = 5.0
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.800000p1
//     v1 = f32const 0x1.000000p2
//     v2 = fmul v0, v0  ; v0 = 0x1.800000p1, v0 = 0x1.800000p1
//     v3 = fmul v1, v1  ; v1 = 0x1.000000p2, v1 = 0x1.000000p2
//     v4 = fadd v2, v3
//     v5 = sqrt v4
//     return v5
//
// block1:
//     v6 = f32const 0.0
//     return v6  ; v6 = 0.0
// }
// run: ~= 5 (tolerance: 0.01)
