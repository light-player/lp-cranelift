// test run
// test fixed32

float main() {
    float a = 2.0;
    float b = 3.0;
    float c = 4.0;
    return (a + b) * c - 1.5;
}

// run: ~= 18.5

// function u0:0() -> i32 system_v {
// block0:
//     v8 = iconst.i32 0x0002_0000
//     v9 = iconst.i32 0x0003_0000
//     v10 = iconst.i32 0x0004_0000
//     v11 = iadd v8, v9  ; v8 = 0x0002_0000, v9 = 0x0003_0000
//     v12 = sextend.i64 v11
//     v13 = sextend.i64 v10  ; v10 = 0x0004_0000
//     v14 = imul v12, v13
//     v15 = iconst.i64 16
//     v16 = sshr v14, v15  ; v15 = 16
//     v17 = ireduce.i32 v16
//     v18 = iconst.i32 0x0001_8000
//     v19 = isub v17, v18  ; v18 = 0x0001_8000
//     return v19
//
// block1:
//     v20 = iconst.i32 0
//     return v20  ; v20 = 0
// }
