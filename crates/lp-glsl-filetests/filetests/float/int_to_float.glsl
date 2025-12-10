// test compile
// test run
// target riscv32.fixed32

float main() {
    int x = 5;
    float y = 2.5;
    return x + y;  // x implicitly converted to float
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = iconst.i32 5
//     v1 = f32const 0x1.400000p1
//     v2 = fcvt_from_sint.f32 v0  ; v0 = 5
//     v3 = fadd v2, v1  ; v1 = 0x1.400000p1
//     return v3
//
// block1:
//     v4 = f32const 0.0
//     return v4  ; v4 = 0.0
// }
// run: ~= 7.5 (tolerance: 0.01)
