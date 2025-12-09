// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return radians(180.0);  // Should be approximately π
}

// function u0:0() -> i32 system_v {
// block0:
//     v4 = iconst.i32 0x00b4_0000
//     v5 = iconst.i32 1144
//     v6 = sextend.i64 v4  ; v4 = 0x00b4_0000
//     v7 = sextend.i64 v5  ; v5 = 1144
//     v8 = imul v6, v7
//     v9 = iconst.i64 16
//     v10 = sshr v8, v9  ; v9 = 16
//     v11 = ireduce.i32 v10
//     return v11
//
// block1:
//     v12 = iconst.i32 0
//     return v12  ; v12 = 0
// }
// run: ~= 3.1415927 
