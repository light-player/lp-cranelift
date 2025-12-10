// test riscv32.fixed32
// test run

float main() {
    return 2.5 * 4.0;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.400000p1
//     v1 = f32const 0x1.000000p2
//     v2 = fmul v0, v1  ; v0 = 0x1.400000p1, v1 = 0x1.000000p2
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
//     v0 = iconst.i32 0x0002_8000
//     v1 = iconst.i32 0x0004_0000
//     v2 = sextend.i64 v0  ; v0 = 0x0002_8000
//     v3 = sextend.i64 v1  ; v1 = 0x0004_0000
//     v4 = imul v2, v3
//     v5 = iconst.i64 16
//     v6 = sshr v4, v5  ; v5 = 16
//     v7 = ireduce.i32 v6
//     return v7
//
// block1:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
// }
// run: ≈ 10.0
