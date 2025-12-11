// test riscv32.fixed32
// test run

vec3 main() {
    return vec3(1.0, 2.0, 3.0) + vec3(0.5, 1.5, 2.5);
}

// Generated CLIF
// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p-1
//     v5 = f32const 0x1.800000p0
//     v6 = f32const 0x1.400000p1
//     v7 = fadd v1, v4  ; v1 = 0x1.000000p0, v4 = 0x1.000000p-1
//     v8 = fadd v2, v5  ; v2 = 0x1.000000p1, v5 = 0x1.800000p0
//     v9 = fadd v3, v6  ; v3 = 0x1.800000p1, v6 = 0x1.400000p1
//     store notrap aligned v7, v0
//     store notrap aligned v8, v0+4
//     store notrap aligned v9, v0+8
//     return
//
// block1:
//     v10 = f32const 0.0
//     store notrap aligned v10, v0  ; v10 = 0.0
//     v11 = f32const 0.0
//     store notrap aligned v11, v0+4  ; v11 = 0.0
//     v12 = f32const 0.0
//     store notrap aligned v12, v0+8  ; v12 = 0.0
//     return
// }
//
// Transformed CLIF
// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = iconst.i32 0x8000
//     v5 = iconst.i32 0x0001_8000
//     v6 = iconst.i32 0x0002_8000
//     v7 = iadd v1, v4  ; v1 = 0x0001_0000, v4 = 0x8000
//     v8 = iadd v2, v5  ; v2 = 0x0002_0000, v5 = 0x0001_8000
//     v9 = iadd v3, v6  ; v3 = 0x0003_0000, v6 = 0x0002_8000
//     store notrap aligned v7, v0
//     store notrap aligned v8, v0+4
//     store notrap aligned v9, v0+8
//     return
//
// block1:
//     v10 = iconst.i32 0
//     store notrap aligned v10, v0  ; v10 = 0
//     v11 = iconst.i32 0
//     store notrap aligned v11, v0+4  ; v11 = 0
//     v12 = iconst.i32 0
//     store notrap aligned v12, v0+8  ; v12 = 0
//     return
// }
// run: ≈ vec3(1.5, 3.5, 5.5)
