// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return sqrt(16.0);  // 4.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v3 = iconst.i32 0x0010_0000
//     v4 = iconst.i32 0
//     v5 = icmp eq v3, v4  ; v3 = 0x0010_0000, v4 = 0
//     v6 = iconst.i32 8
//     v7 = sshr v3, v6  ; v3 = 0x0010_0000, v6 = 8
//     v8 = sextend.i64 v3  ; v3 = 0x0010_0000
//     v9 = sextend.i64 v7
//     v10 = ishl v8, v6  ; v6 = 8
//     v11 = sdiv v10, v9
//     v12 = iadd v9, v11
//     v13 = iconst.i64 1
//     v14 = sshr v12, v13  ; v13 = 1
//     v15 = sdiv v10, v14
//     v16 = iadd v14, v15
//     v17 = iconst.i64 1
//     v18 = sshr v16, v17  ; v17 = 1
//     v19 = sdiv v10, v18
//     v20 = iadd v18, v19
//     v21 = iconst.i64 1
//     v22 = sshr v20, v21  ; v21 = 1
//     v23 = sdiv v10, v22
//     v24 = iadd v22, v23
//     v25 = iconst.i64 1
//     v26 = sshr v24, v25  ; v25 = 1
//     v27 = sdiv v10, v26
//     v28 = iadd v26, v27
//     v29 = iconst.i64 1
//     v30 = sshr v28, v29  ; v29 = 1
//     v31 = ireduce.i32 v30
//     v32 = select v5, v4, v31  ; v4 = 0
//     return v32
//
// block1:
//     v33 = iconst.i32 0
//     return v33  ; v33 = 0
// }
// run: ~= 4
