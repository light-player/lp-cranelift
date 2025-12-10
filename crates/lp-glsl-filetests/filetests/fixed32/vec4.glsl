// test riscv32.fixed32
// test run

vec4 main() {
    return vec4(1.0, 2.0, 3.0, 4.0) + vec4(0.5, 1.5, 2.5, 3.5);
}

// Generated CLIF
// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = f32const 0x1.000000p-1
//     v6 = f32const 0x1.800000p0
//     v7 = f32const 0x1.400000p1
//     v8 = f32const 0x1.c00000p1
//     v9 = fadd v1, v5  ; v1 = 0x1.000000p0, v5 = 0x1.000000p-1
//     v10 = fadd v2, v6  ; v2 = 0x1.000000p1, v6 = 0x1.800000p0
//     v11 = fadd v3, v7  ; v3 = 0x1.800000p1, v7 = 0x1.400000p1
//     v12 = fadd v4, v8  ; v4 = 0x1.000000p2, v8 = 0x1.c00000p1
//     store notrap aligned v9, v0
//     store notrap aligned v10, v0+4
//     store notrap aligned v11, v0+8
//     store notrap aligned v12, v0+12
//     return
//
// block1:
//     v13 = f32const 0.0
//     store notrap aligned v13, v0  ; v13 = 0.0
//     v14 = f32const 0.0
//     store notrap aligned v14, v0+4  ; v14 = 0.0
//     v15 = f32const 0.0
//     store notrap aligned v15, v0+8  ; v15 = 0.0
//     v16 = f32const 0.0
//     store notrap aligned v16, v0+12  ; v16 = 0.0
//     return
// }
//
// Transformed CLIF
// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x0004_0000
//     v5 = iconst.i32 0x8000
//     v6 = iconst.i32 0x0001_8000
//     v7 = iconst.i32 0x0002_8000
//     v8 = iconst.i32 0x0003_8000
//     v9 = iadd v1, v5  ; v1 = 0x0001_0000, v5 = 0x8000
//     v10 = iadd v2, v6  ; v2 = 0x0002_0000, v6 = 0x0001_8000
//     v11 = iadd v3, v7  ; v3 = 0x0003_0000, v7 = 0x0002_8000
//     v12 = iadd v4, v8  ; v4 = 0x0004_0000, v8 = 0x0003_8000
//     store notrap aligned v9, v0
//     store notrap aligned v10, v0+4
//     store notrap aligned v11, v0+8
//     store notrap aligned v12, v0+12
//     return
//
// block1:
//     v13 = iconst.i32 0
//     store notrap aligned v13, v0  ; v13 = 0
//     v14 = iconst.i32 0
//     store notrap aligned v14, v0+4  ; v14 = 0
//     v15 = iconst.i32 0
//     store notrap aligned v15, v0+8  ; v15 = 0
//     v16 = iconst.i32 0
//     store notrap aligned v16, v0+12  ; v16 = 0
//     return
// }
// run: ≈ vec4(1.5, 3.5, 5.5, 7.5)
