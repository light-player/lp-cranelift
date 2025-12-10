// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return floor(3.7);  // Should return 3.0
}

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
// run: ~= 3.0
