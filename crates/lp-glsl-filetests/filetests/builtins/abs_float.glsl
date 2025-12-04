// test compile
// test run

float main() {
    return abs(-3.5);  // 3.5
}

// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.c00000p1
//     v1 = fneg v0  ; v0 = 0x1.c00000p1
//     v2 = fabs v1
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 0 (tolerance: 0.01)
