// test compile

// target riscv32.fixed32
int main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(2.0, 2.0, 2.0);
    vec3 result = (a + b) * 2.0;  // ((3.0, 4.0, 5.0) * 2.0) = (6.0, 8.0, 10.0)
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0001_0000
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 0x0003_0000
//     v3 = iconst.i32 0x0002_0000
//     v4 = iconst.i32 0x0002_0000
//     v5 = iconst.i32 0x0002_0000
//     v6 = iadd v0, v3  ; v0 = 0x0001_0000, v3 = 0x0002_0000
//     v7 = iadd v1, v4  ; v1 = 0x0002_0000, v4 = 0x0002_0000
//     v8 = iadd v2, v5  ; v2 = 0x0003_0000, v5 = 0x0002_0000
//     v9 = iconst.i32 0x0002_0000
//     v10 = sextend.i64 v6
//     v11 = sextend.i64 v9  ; v9 = 0x0002_0000
//     v12 = imul v10, v11
//     v13 = iconst.i64 16
//     v14 = sshr v12, v13  ; v13 = 16
//     v15 = ireduce.i32 v14
//     v16 = sextend.i64 v7
//     v17 = sextend.i64 v9  ; v9 = 0x0002_0000
//     v18 = imul v16, v17
//     v19 = iconst.i64 16
//     v20 = sshr v18, v19  ; v19 = 16
//     v21 = ireduce.i32 v20
//     v22 = sextend.i64 v8
//     v23 = sextend.i64 v9  ; v9 = 0x0002_0000
//     v24 = imul v22, v23
//     v25 = iconst.i64 16
//     v26 = sshr v24, v25  ; v25 = 16
//     v27 = ireduce.i32 v26
//     v28 = iconst.i32 1
//     return v28  ; v28 = 1
//
// block1:
//     v29 = iconst.i32 0
//     return v29  ; v29 = 0
// }


