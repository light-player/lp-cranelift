// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    return floor(vec2(3.7, -2.3));  // Should return (3.0, -3.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v8 = iconst.i32 0x0003_b333
//     v9 = iconst.i32 0x0002_4ccd
//     v10 = ineg v9  ; v9 = 0x0002_4ccd
//     v11 = iconst.i64 16
//     v12 = sextend.i64 v8  ; v8 = 0x0003_b333
//     v13 = sshr v12, v11  ; v11 = 16
//     v14 = ishl v13, v11  ; v11 = 16
//     v15 = ireduce.i32 v14
//     v16 = iconst.i64 16
//     v17 = sextend.i64 v10
//     v18 = sshr v17, v16  ; v16 = 16
//     v19 = ishl v18, v16  ; v16 = 16
//     v20 = ireduce.i32 v19
//     store notrap aligned v15, v0
//     store notrap aligned v20, v0+4
//     return
//
// block1:
//     v21 = iconst.i32 0
//     store notrap aligned v21, v0  ; v21 = 0
//     v22 = iconst.i32 0
//     store notrap aligned v22, v0+4  ; v22 = 0
//     return
// }
// run: ≈ vec2(3.0, -3.0) (tolerance: 0.01)
