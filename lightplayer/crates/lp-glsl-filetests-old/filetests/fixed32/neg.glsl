// test riscv32.fixed32
// test run

float main() {
    return -5.5;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.600000p2
//     v1 = fneg v0  ; v0 = 0x1.600000p2
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0005_8000
//     v1 = ineg v0  ; v0 = 0x0005_8000
//     return v1
//
// block1:
//     v2 = iconst.i32 0
//     return v2  ; v2 = 0
// }
// run: ≈ -5.5
