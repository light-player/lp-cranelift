// test riscv32.fixed32
// test run

float main() {
    return 0.0 / 0.0;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0.0
//     v2 = fdiv v0, v1  ; v0 = 0.0, v1 = 0.0
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     v2 = iconst.i32 0
//     v3 = icmp eq v1, v2  ; v1 = 0, v2 = 0
//     v4 = iconst.i32 0x7fff_0000
//     v5 = iconst.i32 -2147483648
//     v6 = icmp eq v0, v2  ; v0 = 0, v2 = 0
//     v7 = icmp slt v0, v2  ; v0 = 0, v2 = 0
//     v8 = select v7, v5, v4  ; v5 = -2147483648, v4 = 0x7fff_0000
//     v9 = select v6, v2, v8  ; v2 = 0
//     v10 = iconst.i32 1
//     v11 = select v3, v10, v1  ; v10 = 1, v1 = 0
//     v12 = sextend.i64 v0  ; v0 = 0
//     v13 = iconst.i64 16
//     v14 = ishl v12, v13  ; v13 = 16
//     v15 = sextend.i64 v11
//     v16 = sdiv v14, v15
//     v17 = ireduce.i32 v16
//     v18 = select v3, v9, v17
//     return v18
//
// block1:
//     v19 = iconst.i32 0
//     return v19  ; v19 = 0
// }
// run: ≈ 0.0
