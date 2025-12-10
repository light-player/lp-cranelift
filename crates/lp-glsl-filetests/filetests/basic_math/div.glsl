// test run
// test fixed32

float main() {
    float a = 10.0;
    float b = 4.0;
    return a / b;
}

// run: ~= 2.5

// function u0:0() -> i32 system_v {
// block0:
//     v4 = iconst.i32 0x000a_0000
//     v5 = iconst.i32 0x0004_0000
//     v6 = sextend.i64 v4  ; v4 = 0x000a_0000
//     v7 = iconst.i64 16
//     v8 = ishl v6, v7  ; v7 = 16
//     v9 = sextend.i64 v5  ; v5 = 0x0004_0000
//     v10 = sdiv v8, v9
//     v11 = ireduce.i32 v10
//     return v11
//
// block1:
//     v12 = iconst.i32 0
//     return v12  ; v12 = 0
// }
