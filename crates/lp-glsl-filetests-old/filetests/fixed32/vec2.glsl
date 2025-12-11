// test riscv32.fixed32
// test run

vec2 main() {
    return vec2(10, 20) + vec2(30, 40);
}

// Generated CLIF
// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 10
//     v2 = iconst.i32 20
//     v3 = fcvt_from_sint.f32 v1  ; v1 = 10
//     v4 = fcvt_from_sint.f32 v2  ; v2 = 20
//     v5 = iconst.i32 30
//     v6 = iconst.i32 40
//     v7 = fcvt_from_sint.f32 v5  ; v5 = 30
//     v8 = fcvt_from_sint.f32 v6  ; v6 = 40
//     v9 = fadd v3, v7
//     v10 = fadd v4, v8
//     store notrap aligned v9, v0
//     store notrap aligned v10, v0+4
//     return
//
// block1:
//     v11 = f32const 0.0
//     store notrap aligned v11, v0  ; v11 = 0.0
//     v12 = f32const 0.0
//     store notrap aligned v12, v0+4  ; v12 = 0.0
//     return
// }
//
// Transformed CLIF
// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 10
//     v2 = iconst.i32 20
//     v3 = iconst.i32 16
//     v4 = ishl v1, v3  ; v1 = 10, v3 = 16
//     v5 = iconst.i32 16
//     v6 = ishl v2, v5  ; v2 = 20, v5 = 16
//     v7 = iconst.i32 30
//     v8 = iconst.i32 40
//     v9 = iconst.i32 16
//     v10 = ishl v7, v9  ; v7 = 30, v9 = 16
//     v11 = iconst.i32 16
//     v12 = ishl v8, v11  ; v8 = 40, v11 = 16
//     v13 = iadd v4, v10
//     v14 = iadd v6, v12
//     store notrap aligned v13, v0
//     store notrap aligned v14, v0+4
//     return
//
// block1:
//     v15 = iconst.i32 0
//     store notrap aligned v15, v0  ; v15 = 0
//     v16 = iconst.i32 0
//     store notrap aligned v16, v0+4  ; v16 = 0
//     return
// }
// run: ≈ vec2(40.0, 60.0)
