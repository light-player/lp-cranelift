// test compile
// test fixed32

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// function u0:0() -> i32 fast {
// block0:
//     v4 = iconst.i32 0x0002_0000
//     v5 = iconst.i32 0x0003_8000
//     v6 = sextend.i64 v4  ; v4 = 0x0002_0000
//     v7 = sextend.i64 v5  ; v5 = 0x0003_8000
//     v8 = imul v6, v7
//     v9 = iconst.i64 16
//     v10 = sshr v8, v9  ; v9 = 16
//     v11 = ireduce.i32 v10
//     return v11
//
// block1:
//     v12 = iconst.i32 0
//     return v12  ; v12 = 0
// }
