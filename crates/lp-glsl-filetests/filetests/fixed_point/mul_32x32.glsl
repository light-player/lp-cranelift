// test compile
// test fixed64

float main() {
    float a = 2.0;
    float b = 3.5;
    return a * b;
}

// function u0:0() -> i64 apple_aarch64 {
// block0:
//     v4 = iconst.i64 0x0002_0000_0000
//     v5 = iconst.i64 0x0003_8000_0000
//     v6 = sextend.i128 v4  ; v4 = 0x0002_0000_0000
//     v7 = sextend.i128 v5  ; v5 = 0x0003_8000_0000
//     v8 = imul v6, v7
//     v9 = iconst.i64 32
//     v10 = sshr v8, v9  ; v9 = 32
//     v11 = ireduce.i64 v10
//     return v11
//
// block1:
//     v12 = iconst.i64 0
//     return v12  ; v12 = 0
// }
