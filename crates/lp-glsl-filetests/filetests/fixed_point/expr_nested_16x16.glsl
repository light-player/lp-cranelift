// test compile
// test fixed32

float main() {
    float x = 1.5;
    float y = 2.5;
    return ((x * 2.0) + (y / 2.0)) - 0.5;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v10 = iconst.i32 0x0001_8000
//     v11 = iconst.i32 0x0002_8000
//     v12 = iconst.i32 0x0002_0000
//     v13 = sextend.i64 v10  ; v10 = 0x0001_8000
//     v14 = sextend.i64 v12  ; v12 = 0x0002_0000
//     v15 = imul v13, v14
//     v16 = iconst.i64 16
//     v17 = sshr v15, v16  ; v16 = 16
//     v18 = ireduce.i32 v17
//     v19 = iconst.i32 0x0002_0000
//     v20 = sextend.i64 v11  ; v11 = 0x0002_8000
//     v21 = iconst.i64 16
//     v22 = ishl v20, v21  ; v21 = 16
//     v23 = sextend.i64 v19  ; v19 = 0x0002_0000
//     v24 = sdiv v22, v23
//     v25 = ireduce.i32 v24
//     v26 = iadd v18, v25
//     v27 = iconst.i32 0x8000
//     v28 = isub v26, v27  ; v27 = 0x8000
//     return v28
//
// block1:
//     v29 = iconst.i32 0
//     return v29  ; v29 = 0
// }
