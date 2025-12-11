// test compile
// test run
// target riscv32.fixed32

float main() {
    int a = 3;
    float b = 2.0;
    float c = a * b;  // 3 → 3.0, then 3.0 * 2.0
    return c;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 3
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 16
//     v3 = ishl v0, v2  ; v0 = 3, v2 = 16
//     v4 = sextend.i64 v3
//     v5 = sextend.i64 v1  ; v1 = 0x0002_0000
//     v6 = imul v4, v5
//     v7 = iconst.i64 16
//     v8 = sshr v6, v7  ; v7 = 16
//     v9 = ireduce.i32 v8
//     return v9
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }
// run: ~= 6 (tolerance: 0.01)
