// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = step(5.0, 10.0);  // x(10.0) >= edge(5.0), returns 1.0
    return result > 0.99;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0x0005_0000
//     v1 = iconst.i32 0x000a_0000
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0x0001_0000
//     v4 = icmp slt v1, v0  ; v1 = 0x000a_0000, v0 = 0x0005_0000
//     v5 = sextend.i32 v4
//     v6 = select v5, v2, v3  ; v2 = 0, v3 = 0x0001_0000
//     v7 = iconst.i32 0xfd71
//     v8 = icmp sgt v6, v7  ; v7 = 0xfd71
//     v9 = sextend.i32 v8
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v9, v10, v11  ; v10 = 1, v11 = 0
//     return v12
//
// block1:
//     v13 = iconst.i8 0
//     return v13  ; v13 = 0
// }
// run: == true
