// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return abs(-3.5);  // 3.5
}

// function u0:0() -> i32 system_v {
// block0:
//     v4 = iconst.i32 0x0003_8000
//     v5 = ineg v4  ; v4 = 0x0003_8000
//     v6 = iabs v5
//     return v6
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
// run: ~= 3.5
