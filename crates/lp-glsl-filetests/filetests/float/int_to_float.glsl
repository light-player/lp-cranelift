// test compile
// test run
// target riscv32.fixed32

float main() {
    int x = 5;
    float y = 2.5;
    return x + y;  // x implicitly converted to float
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 0x0002_8000
//     v2 = iconst.i32 16
//     v3 = ishl v0, v2  ; v0 = 5, v2 = 16
//     v4 = iadd v3, v1  ; v1 = 0x0002_8000
//     return v4
//
// block1:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
// run: ~= 7.5 (tolerance: 0.01)
