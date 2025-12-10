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
//     v0 = iconst.i32 0x0001_0000
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 0x0003_0000
//     v3 = iconst.i32 0x0004_0000
//     v4 = sextend.i64 v0  ; v0 = 0x0001_0000
//     v5 = sextend.i64 v3  ; v3 = 0x0004_0000
//     v6 = imul v4, v5
//     v7 = iconst.i64 16
//     v8 = sshr v6, v7  ; v7 = 16
//     v9 = ireduce.i32 v8
//     v10 = sextend.i64 v2  ; v2 = 0x0003_0000
//     v11 = sextend.i64 v1  ; v1 = 0x0002_0000
//     v12 = imul v10, v11
//     v13 = iconst.i64 16
//     v14 = sshr v12, v13  ; v13 = 16
//     v15 = ireduce.i32 v14
//     v16 = isub v9, v15
//     return v16
//
// block1:
//     v17 = iconst.i32 0
//     return v17  ; v17 = 0
// }
// run: ~= -2.0  // 1*4 - 2*3 = 4 - 6 = -2
