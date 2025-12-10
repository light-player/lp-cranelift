// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    vec2 deg = vec2(90.0, 45.0);
    return radians(deg);
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v8 = iconst.i32 0x005a_0000
//     v9 = iconst.i32 0x002d_0000
//     v10 = iconst.i32 1144
//     v11 = sextend.i64 v8  ; v8 = 0x005a_0000
//     v12 = sextend.i64 v10  ; v10 = 1144
//     v13 = imul v11, v12
//     v14 = iconst.i64 16
//     v15 = sshr v13, v14  ; v14 = 16
//     v16 = ireduce.i32 v15
//     v17 = sextend.i64 v9  ; v9 = 0x002d_0000
//     v18 = sextend.i64 v10  ; v10 = 1144
//     v19 = imul v17, v18
//     v20 = iconst.i64 16
//     v21 = sshr v19, v20  ; v20 = 16
//     v22 = ireduce.i32 v21
//     store notrap aligned v16, v0
//     store notrap aligned v22, v0+4
//     return
//
// block1:
//     v23 = iconst.i32 0
//     store notrap aligned v23, v0  ; v23 = 0
//     v24 = iconst.i32 0
//     store notrap aligned v24, v0+4  ; v24 = 0
//     return
// }
// run: ≈ vec2(1.5708, 0.7854) (tolerance: 0.01)
