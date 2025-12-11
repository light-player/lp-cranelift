// test riscv32.fixed32
// test run

float main() {
    return abs(-7.5);
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.e00000p2
//     v1 = fneg v0  ; v0 = 0x1.e00000p2
//     v2 = fabs v1
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
//     v0 = iconst.i32 0x0007_8000
//     v1 = ineg v0  ; v0 = 0x0007_8000
//     v2 = iconst.i32 0
//     v3 = icmp slt v1, v2  ; v2 = 0
//     v4 = ineg v1
//     v5 = select v3, v4, v1
//     return v5
//
// block1:
//     v6 = iconst.i32 0
//     return v6  ; v6 = 0
// }
// run: ≈ 7.5
