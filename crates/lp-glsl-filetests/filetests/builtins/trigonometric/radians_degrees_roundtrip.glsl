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
//     v6 = iconst.i32 0x00b4_0000
//     v7 = iconst.i32 1144
//     v8 = sextend.i64 v6  ; v6 = 0x00b4_0000
//     v9 = sextend.i64 v7  ; v7 = 1144
//     v10 = imul v8, v9
//     v11 = iconst.i64 16
//     v12 = sshr v10, v11  ; v11 = 16
//     v13 = ireduce.i32 v12
//     v14 = iconst.i32 0x0039_4bb8
//     v15 = sextend.i64 v13
//     v16 = sextend.i64 v14  ; v14 = 0x0039_4bb8
//     v17 = imul v15, v16
//     v18 = iconst.i64 16
//     v19 = sshr v17, v18  ; v18 = 16
//     v20 = ireduce.i32 v19
//     return v20
//
// block1:
//     v21 = iconst.i32 0
//     return v21  ; v21 = 0
// }
// run: ~= 180 
