// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return clamp(7.0, 2.0, 5.0);  // 5.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v6 = iconst.i32 0x0007_0000
//     v7 = iconst.i32 0x0002_0000
//     v8 = iconst.i32 0x0005_0000
//     v9 = icmp sge v6, v7  ; v6 = 0x0007_0000, v7 = 0x0002_0000
//     v10 = select v9, v6, v7  ; v6 = 0x0007_0000, v7 = 0x0002_0000
//     v11 = icmp sle v10, v8  ; v8 = 0x0005_0000
//     v12 = select v11, v10, v8  ; v8 = 0x0005_0000
//     return v12
//
// block1:
//     v13 = iconst.i32 0
//     return v13  ; v13 = 0
// }
// run: ~= 5
