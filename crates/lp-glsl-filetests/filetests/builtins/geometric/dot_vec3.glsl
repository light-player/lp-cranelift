// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    return dot(a, b);  // 1*4 + 2*5 + 3*6 = 32.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0001_0000
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 0x0003_0000
//     v3 = iconst.i32 0x0004_0000
//     v4 = iconst.i32 0x0005_0000
//     v5 = iconst.i32 0x0006_0000
//     v6 = sextend.i64 v0  ; v0 = 0x0001_0000
//     v7 = sextend.i64 v3  ; v3 = 0x0004_0000
//     v8 = imul v6, v7
//     v9 = iconst.i64 16
//     v10 = sshr v8, v9  ; v9 = 16
//     v11 = ireduce.i32 v10
//     v12 = sextend.i64 v1  ; v1 = 0x0002_0000
//     v13 = sextend.i64 v4  ; v4 = 0x0005_0000
//     v14 = imul v12, v13
//     v15 = iconst.i64 16
//     v16 = sshr v14, v15  ; v15 = 16
//     v17 = ireduce.i32 v16
//     v18 = iadd v11, v17
//     v19 = sextend.i64 v2  ; v2 = 0x0003_0000
//     v20 = sextend.i64 v5  ; v5 = 0x0006_0000
//     v21 = imul v19, v20
//     v22 = iconst.i64 16
//     v23 = sshr v21, v22  ; v22 = 16
//     v24 = ireduce.i32 v23
//     v25 = iadd v18, v24
//     return v25
//
// block1:
//     v26 = iconst.i32 0
//     return v26  ; v26 = 0
// }
// run: ~= 32
