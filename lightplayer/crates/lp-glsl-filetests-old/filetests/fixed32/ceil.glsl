// test riscv32.fixed32
// test run

float main() {
    return ceil(3.7);
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.d9999ap1
//     v1 = ceil v0  ; v0 = 0x1.d9999ap1
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
//     v0 = iconst.i32 0x0003_b333
//     v1 = iconst.i32 0xffff
//     v2 = iadd v0, v1  ; v0 = 0x0003_b333, v1 = 0xffff
//     v3 = iconst.i32 16
//     v4 = sshr v2, v3  ; v3 = 16
//     v5 = ishl v4, v3  ; v3 = 16
//     return v5
//
// block1:
//     v6 = iconst.i32 0
//     return v6  ; v6 = 0
// }
// run: ≈ 4.0
