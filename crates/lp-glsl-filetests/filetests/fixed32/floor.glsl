// test riscv32.fixed32
// test run

float main() {
    return floor(3.7);
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.d9999ap1
//     v1 = floor v0  ; v0 = 0x1.d9999ap1
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
//     v1 = iconst.i32 16
//     v2 = sshr v0, v1  ; v0 = 0x0003_b333, v1 = 16
//     v3 = ishl v2, v1  ; v1 = 16
//     return v3
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
// run: ≈ 3.0
