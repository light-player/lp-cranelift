// test riscv32.fixed32
// test run

float main() {
    return sqrt(16.0);
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p4
//     v1 = sqrt v0  ; v0 = 0x1.000000p4
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0010_0000
//     v1 = iconst.i32 0
//     v2 = icmp eq v0, v1  ; v0 = 0x0010_0000, v1 = 0
//     v3 = icmp slt v0, v1  ; v0 = 0x0010_0000, v1 = 0
//     v4 = bor v2, v3
//     v5 = sextend.i64 v0  ; v0 = 0x0010_0000
//     v6 = iconst.i64 16
//     v7 = ishl v5, v6  ; v6 = 16
//     v8 = iconst.i64 16
//     v9 = sshr v7, v8  ; v8 = 16
//     v10 = iconst.i64 1
//     v11 = smax v9, v10  ; v10 = 1
//     v12 = sdiv v7, v11
//     v13 = iadd v11, v12
//     v14 = iconst.i64 1
//     v15 = sshr v13, v14  ; v14 = 1
//     v16 = sdiv v7, v15
//     v17 = iadd v15, v16
//     v18 = sshr v17, v14  ; v14 = 1
//     v19 = sdiv v7, v18
//     v20 = iadd v18, v19
//     v21 = sshr v20, v14  ; v14 = 1
//     v22 = sdiv v7, v21
//     v23 = iadd v21, v22
//     v24 = sshr v23, v14  ; v14 = 1
//     v25 = ireduce.i32 v24
//     v26 = select v4, v1, v25  ; v1 = 0
//     return v26
//
// block1:
//     v27 = iconst.i32 0
//     return v27  ; v27 = 0
// }
// run: ≈ 4.0


