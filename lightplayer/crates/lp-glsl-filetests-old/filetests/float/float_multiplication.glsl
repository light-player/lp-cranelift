// test compile
// test run
// target riscv32.fixed32

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0002_0000
//     v1 = iconst.i32 0x0003_8000
//     v2 = sextend.i64 v0  ; v0 = 0x0002_0000
//     v3 = sextend.i64 v1  ; v1 = 0x0003_8000
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
// run: ~= 7 (tolerance: 0.01)
