// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return ceil(3.2);  // Should return 4.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0003_3333
//     v1 = iconst.i32 0xffff
//     v2 = iadd v0, v1  ; v0 = 0x0003_3333, v1 = 0xffff
//     v3 = iconst.i32 16
//     v4 = sshr v2, v3  ; v3 = 16
//     v5 = ishl v4, v3  ; v3 = 16
//     return v5
//
// block1:
//     v6 = iconst.i32 0
//     return v6  ; v6 = 0
// }
// run: ~= 4.0
