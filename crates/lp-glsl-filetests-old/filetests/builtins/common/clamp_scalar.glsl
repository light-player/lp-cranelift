// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return clamp(7.0, 2.0, 5.0);  // 5.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0007_0000
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 0x0005_0000
//     v3 = icmp sgt v0, v1  ; v0 = 0x0007_0000, v1 = 0x0002_0000
//     v4 = select v3, v0, v1  ; v0 = 0x0007_0000, v1 = 0x0002_0000
//     v5 = icmp slt v4, v2  ; v2 = 0x0005_0000
//     v6 = select v5, v4, v2  ; v2 = 0x0005_0000
//     return v6
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
// run: ~= 5
