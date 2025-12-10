// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    return abs(vec2(-1.5, 2.3));  // Should return (1.5, 2.3)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v8 = iconst.i32 0x0001_8000
//     v9 = ineg v8  ; v8 = 0x0001_8000
//     v10 = iconst.i32 0x0002_4ccd
//     v11 = iabs v9
//     v12 = iabs v10  ; v10 = 0x0002_4ccd
//     store notrap aligned v11, v0
//     store notrap aligned v12, v0+4
//     return
//
// block1:
//     v13 = iconst.i32 0
//     store notrap aligned v13, v0  ; v13 = 0
//     v14 = iconst.i32 0
//     store notrap aligned v14, v0+4  ; v14 = 0
//     return
// }
// run: ≈ vec2(1.5, 2.3) (tolerance: 0.01)
