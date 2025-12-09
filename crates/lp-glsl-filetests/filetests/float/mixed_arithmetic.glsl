// test compile
// test run
// target riscv32

float main() {
    int a = 3;
    float b = 2.0;
    float c = a * b;  // 3 → 3.0, then 3.0 * 2.0
    return c;
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = iconst.i32 3
//     v1 = f32const 0x1.000000p1
//     v2 = fcvt_from_sint.f32 v0  ; v0 = 3
//     v3 = fmul v2, v1  ; v1 = 0x1.000000p1
//     return v3
//
// block1:
//     v4 = f32const 0.0
//     return v4  ; v4 = 0.0
// }
// run: ~= 6 (tolerance: 0.01)
