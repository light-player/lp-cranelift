// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return abs(-3.5);  // 3.5
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0003_8000
//     v1 = ineg v0  ; v0 = 0x0003_8000
//     v2 = iconst.i32 0
//     v3 = icmp slt v1, v2  ; v2 = 0
//     v4 = ineg v1
//     v5 = select v3, v4, v1
//     return v5
//
// block1:
//     v6 = iconst.i32 0
//     return v6  ; v6 = 0
// }
// run: ~= 3.5
