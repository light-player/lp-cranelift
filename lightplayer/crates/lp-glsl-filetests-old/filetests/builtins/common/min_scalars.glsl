// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return min(5.0, 3.0);  // 3.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0005_0000
//     v1 = iconst.i32 0x0003_0000
//     v2 = icmp slt v0, v1  ; v0 = 0x0005_0000, v1 = 0x0003_0000
//     v3 = select v2, v0, v1  ; v0 = 0x0005_0000, v1 = 0x0003_0000
//     return v3
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
// run: ~= 3
