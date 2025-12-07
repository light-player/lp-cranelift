// test compile
// test run

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p1
//     v1 = f32const 0x1.c00000p1
//     v2 = fmul v0, v1  ; v0 = 0x1.000000p1, v1 = 0x1.c00000p1
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 7 (tolerance: 0.01)
