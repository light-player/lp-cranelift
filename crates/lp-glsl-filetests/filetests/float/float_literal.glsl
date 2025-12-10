// test compile
// test run
// target riscv32.fixed32

float main() {
    return 3.14;
}

// function u0:0() -> i32 system_v {
// block0:
//     v2 = iconst.i32 0x0003_23d7
//     return v2  ; v2 = 0x0003_23d7
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ~= 3.14 (tolerance: 0.01)
