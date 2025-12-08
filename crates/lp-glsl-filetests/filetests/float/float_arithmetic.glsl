// test compile
// test run

float main() {
    float a = 2.5;
    float b = 1.5;
    return a + b;
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.400000p1
//     v1 = f32const 0x1.800000p0
//     v2 = fadd v0, v1  ; v0 = 0x1.400000p1, v1 = 0x1.800000p0
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 4 (tolerance: 0.01)
