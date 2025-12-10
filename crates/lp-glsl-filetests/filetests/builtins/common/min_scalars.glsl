// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return min(5.0, 3.0);  // 3.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v4 = iconst.i32 0x0005_0000
//     v5 = iconst.i32 0x0003_0000
//     v6 = icmp sle v4, v5  ; v4 = 0x0005_0000, v5 = 0x0003_0000
//     v7 = select v6, v4, v5  ; v4 = 0x0005_0000, v5 = 0x0003_0000
//     return v7
//
// block1:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
// }
// run: ~= 3
