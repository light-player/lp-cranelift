// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    return floor(vec2(3.7, -2.3));  // Should return (3.0, -3.0)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0003_b333
//     v2 = iconst.i32 0x0002_4ccd
//     v3 = ineg v2  ; v2 = 0x0002_4ccd
//     v4 = iconst.i32 16
//     v5 = sshr v1, v4  ; v1 = 0x0003_b333, v4 = 16
//     v6 = ishl v5, v4  ; v4 = 16
//     v7 = iconst.i32 16
//     v8 = sshr v3, v7  ; v7 = 16
//     v9 = ishl v8, v7  ; v7 = 16
//     store notrap aligned v6, v0
//     store notrap aligned v9, v0+4
//     return
//
// block1:
//     v10 = iconst.i32 0
//     store notrap aligned v10, v0  ; v10 = 0
//     v11 = iconst.i32 0
//     store notrap aligned v11, v0+4  ; v11 = 0
//     return
// }
// run: ≈ vec2(3.0, -3.0) (tolerance: 0.01)
