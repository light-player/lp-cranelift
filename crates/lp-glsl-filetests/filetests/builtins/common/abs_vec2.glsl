// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    return abs(vec2(-1.5, 2.3));  // Should return (1.5, 2.3)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_8000
//     v2 = ineg v1  ; v1 = 0x0001_8000
//     v3 = iconst.i32 0x0002_4ccd
//     v4 = iconst.i32 0
//     v5 = icmp slt v2, v4  ; v4 = 0
//     v6 = ineg v2
//     v7 = select v5, v6, v2
//     v8 = iconst.i32 0
//     v9 = icmp slt v3, v8  ; v3 = 0x0002_4ccd, v8 = 0
//     v10 = ineg v3  ; v3 = 0x0002_4ccd
//     v11 = select v9, v10, v3  ; v3 = 0x0002_4ccd
//     store notrap aligned v7, v0
//     store notrap aligned v11, v0+4
//     return
//
// block1:
//     v12 = iconst.i32 0
//     store notrap aligned v12, v0  ; v12 = 0
//     v13 = iconst.i32 0
//     store notrap aligned v13, v0+4  ; v13 = 0
//     return
// }
// run: ≈ vec2(1.5, 2.3) (tolerance: 0.01)
