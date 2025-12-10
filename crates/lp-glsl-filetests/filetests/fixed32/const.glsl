// test riscv32.fixed32
// test run

float main() {
    return 123.456;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.edd2f2p6
//     return v0  ; v0 = 0x1.edd2f2p6
//
// block1:
//     v1 = f32const 0.0
//     return v1  ; v1 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x007b_74bd
//     return v0  ; v0 = 0x007b_74bd
//
// block1:
//     v1 = iconst.i32 0
//     return v1  ; v1 = 0
// }
// run: ≈ 123.456
