// test compile
// test run
// target riscv32.fixed32

float main() {
    float x = 10;  // int 10 → float conversion
    return x;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 16
//     v2 = ishl v0, v1  ; v0 = 10, v1 = 16
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ~= 10 (tolerance: 0.01)
