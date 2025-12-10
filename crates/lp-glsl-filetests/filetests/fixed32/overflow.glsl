// test riscv32.fixed32
// test run

float main() {
    return 30000.0 + 3000.0;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.d4c000p14
//     v1 = f32const 0x1.770000p11
//     v2 = fadd v0, v1  ; v0 = 0x1.d4c000p14, v1 = 0x1.770000p11
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x7530_0000
//     v1 = iconst.i32 0x0bb8_0000
//     v2 = iadd v0, v1  ; v0 = 0x7530_0000, v1 = 0x0bb8_0000
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ≈ -30536.0
