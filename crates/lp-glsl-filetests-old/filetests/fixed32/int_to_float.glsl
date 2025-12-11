// test riscv32.fixed32
// test run

float main() {
    int i = 42;
    float f = i;
    return f;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = iconst.i32 42
//     v1 = fcvt_from_sint.f32 v0  ; v0 = 42
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
//     v0 = iconst.i32 42
//     v1 = iconst.i32 16
//     v2 = ishl v0, v1  ; v0 = 42, v1 = 16
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ≈ 42.0
