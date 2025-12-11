// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    float deg = 180.0;
    float rad = radians(deg);
    return degrees(rad);
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x00b4_0000
//     v1 = iconst.i32 1144
//     v2 = sextend.i64 v0  ; v0 = 0x00b4_0000
//     v3 = sextend.i64 v1  ; v1 = 1144
//     v4 = imul v2, v3
//     v5 = iconst.i64 16
//     v6 = sshr v4, v5  ; v5 = 16
//     v7 = ireduce.i32 v6
//     v8 = iconst.i32 0x0039_4bb8
//     v9 = sextend.i64 v7
//     v10 = sextend.i64 v8  ; v8 = 0x0039_4bb8
//     v11 = imul v9, v10
//     v12 = iconst.i64 16
//     v13 = sshr v11, v12  ; v12 = 16
//     v14 = ireduce.i32 v13
//     return v14
//
// block1:
//     v15 = iconst.i32 0
//     return v15  ; v15 = 0
// }
// run: ~= 180 
