// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return floor(3.7);  // Should return 3.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v3 = iconst.i32 0x0003_b333
//     v4 = iconst.i64 16
//     v5 = sextend.i64 v3  ; v3 = 0x0003_b333
//     v6 = sshr v5, v4  ; v4 = 16
//     v7 = ishl v6, v4  ; v4 = 16
//     v8 = ireduce.i32 v7
//     return v8
//
// block1:
//     v9 = iconst.i32 0
//     return v9  ; v9 = 0
// }
// run: ~= 3.0
