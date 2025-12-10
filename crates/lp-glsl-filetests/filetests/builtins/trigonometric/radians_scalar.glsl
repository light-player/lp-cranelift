// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return radians(180.0);  // Should be approximately π
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x00b4_0000
//     v1 = iconst.i32 1144
//     v2 = sextend.i64 v0  ; v0 = 0x00b4_0000
//     v3 = sextend.i64 v1  ; v1 = 1144
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
// run: ~= 3.1415927 
