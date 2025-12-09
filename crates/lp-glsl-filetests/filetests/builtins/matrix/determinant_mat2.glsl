// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return determinant(m);
}

// function u0:0() -> i32 system_v {
// block0:
//     v8 = iconst.i32 0x0001_0000
//     v9 = iconst.i32 0x0002_0000
//     v10 = iconst.i32 0x0003_0000
//     v11 = iconst.i32 0x0004_0000
//     v12 = sextend.i64 v8  ; v8 = 0x0001_0000
//     v13 = sextend.i64 v11  ; v11 = 0x0004_0000
//     v14 = imul v12, v13
//     v15 = iconst.i64 16
//     v16 = sshr v14, v15  ; v15 = 16
//     v17 = ireduce.i32 v16
//     v18 = sextend.i64 v10  ; v10 = 0x0003_0000
//     v19 = sextend.i64 v9  ; v9 = 0x0002_0000
//     v20 = imul v18, v19
//     v21 = iconst.i64 16
//     v22 = sshr v20, v21  ; v21 = 16
//     v23 = ireduce.i32 v22
//     v24 = isub v17, v23
//     return v24
//
// block1:
//     v25 = iconst.i32 0
//     return v25  ; v25 = 0
// }
// run: ~= -2.0  // 1*4 - 2*3 = 4 - 6 = -2
