// test compile
// test run
// target riscv32.fixed32

float main() {
    float a = 2.5;
    float b = 1.5;
    return a + b;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0002_8000
//     v1 = iconst.i32 0x0001_8000
//     v2 = iadd v0, v1  ; v0 = 0x0002_8000, v1 = 0x0001_8000
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ~= 4 (tolerance: 0.01)
