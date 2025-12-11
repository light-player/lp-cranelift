// test riscv32.fixed32
// test run

float main() {
    return -30000.0 - 3000.0;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.d4c000p14
//     v1 = fneg v0  ; v0 = 0x1.d4c000p14
//     v2 = f32const 0x1.770000p11
//     v3 = fsub v1, v2  ; v2 = 0x1.770000p11
//     return v3
//
// block1:
//     v4 = f32const 0.0
//     return v4  ; v4 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x7530_0000
//     v1 = ineg v0  ; v0 = 0x7530_0000
//     v2 = iconst.i32 0x0bb8_0000
//     v3 = isub v1, v2  ; v2 = 0x0bb8_0000
//     return v3
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
// run: ≈ 30536.0
