// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = step(5.0, 10.0);  // x(10.0) >= edge(5.0), returns 1.0
    return result > 0.99;
}

// function u0:0() -> i8 system_v {
// block0:
//     v12 = iconst.i32 0x0005_0000
//     v13 = iconst.i32 0x000a_0000
//     v14 = iconst.i32 0
//     v15 = iconst.i32 0x0001_0000
//     v16 = icmp slt v13, v12  ; v13 = 0x000a_0000, v12 = 0x0005_0000
//     v17 = select v16, v14, v15  ; v14 = 0, v15 = 0x0001_0000
//     v18 = iconst.i32 0xfd71
//     v19 = icmp sgt v17, v18  ; v18 = 0xfd71
//     v8 = iconst.i8 1
//     v9 = iconst.i8 0
//     v10 = select v19, v8, v9  ; v8 = 1, v9 = 0
//     return v10
//
// block1:
//     v11 = iconst.i8 0
//     return v11  ; v11 = 0
// }
// run: == true
