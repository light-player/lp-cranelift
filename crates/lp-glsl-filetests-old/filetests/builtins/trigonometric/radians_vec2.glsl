// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    vec2 deg = vec2(90.0, 45.0);
    return radians(deg);
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x005a_0000
//     v2 = iconst.i32 0x002d_0000
//     v3 = iconst.i32 1144
//     v4 = sextend.i64 v1  ; v1 = 0x005a_0000
//     v5 = sextend.i64 v3  ; v3 = 1144
//     v6 = imul v4, v5
//     v7 = iconst.i64 16
//     v8 = sshr v6, v7  ; v7 = 16
//     v9 = ireduce.i32 v8
//     v10 = sextend.i64 v2  ; v2 = 0x002d_0000
//     v11 = sextend.i64 v3  ; v3 = 1144
//     v12 = imul v10, v11
//     v13 = iconst.i64 16
//     v14 = sshr v12, v13  ; v13 = 16
//     v15 = ireduce.i32 v14
//     store notrap aligned v9, v0
//     store notrap aligned v15, v0+4
//     return
//
// block1:
//     v16 = iconst.i32 0
//     store notrap aligned v16, v0  ; v16 = 0
//     v17 = iconst.i32 0
//     store notrap aligned v17, v0+4  ; v17 = 0
//     return
// }
// run: ≈ vec2(1.5708, 0.7854) (tolerance: 0.01)
