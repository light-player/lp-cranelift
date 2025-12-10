// test compile
// test run
// target riscv32.fixed32

vec3 main() {
    return ceil(vec3(3.2, -2.7, 0.1));  // Should return (4.0, -2.0, 1.0)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0003_3333
//     v2 = iconst.i32 0x0002_b333
//     v3 = ineg v2  ; v2 = 0x0002_b333
//     v4 = iconst.i32 6554
//     v5 = iconst.i32 0xffff
//     v6 = iadd v1, v5  ; v1 = 0x0003_3333, v5 = 0xffff
//     v7 = iconst.i32 16
//     v8 = sshr v6, v7  ; v7 = 16
//     v9 = ishl v8, v7  ; v7 = 16
//     v10 = iconst.i32 0xffff
//     v11 = iadd v3, v10  ; v10 = 0xffff
//     v12 = iconst.i32 16
//     v13 = sshr v11, v12  ; v12 = 16
//     v14 = ishl v13, v12  ; v12 = 16
//     v15 = iconst.i32 0xffff
//     v16 = iadd v4, v15  ; v4 = 6554, v15 = 0xffff
//     v17 = iconst.i32 16
//     v18 = sshr v16, v17  ; v17 = 16
//     v19 = ishl v18, v17  ; v17 = 16
//     store notrap aligned v9, v0
//     store notrap aligned v14, v0+4
//     store notrap aligned v19, v0+8
//     return
//
// block1:
//     v20 = iconst.i32 0
//     store notrap aligned v20, v0  ; v20 = 0
//     v21 = iconst.i32 0
//     store notrap aligned v21, v0+4  ; v21 = 0
//     v22 = iconst.i32 0
//     store notrap aligned v22, v0+8  ; v22 = 0
//     return
// }
// run: ≈ vec3(4, -2, 1) (tolerance: 0.01)
