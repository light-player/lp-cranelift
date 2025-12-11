// test compile

// target riscv32.fixed32
int main() {
    vec3 scaled = 3.0 * vec3(1.0, 2.0, 3.0);  // (3.0, 6.0, 9.0)
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0003_0000
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0x0002_0000
//     v3 = iconst.i32 0x0003_0000
//     v4 = sextend.i64 v0  ; v0 = 0x0003_0000
//     v5 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v6 = imul v4, v5
//     v7 = iconst.i64 16
//     v8 = sshr v6, v7  ; v7 = 16
//     v9 = ireduce.i32 v8
//     v10 = sextend.i64 v0  ; v0 = 0x0003_0000
//     v11 = sextend.i64 v2  ; v2 = 0x0002_0000
//     v12 = imul v10, v11
//     v13 = iconst.i64 16
//     v14 = sshr v12, v13  ; v13 = 16
//     v15 = ireduce.i32 v14
//     v16 = sextend.i64 v0  ; v0 = 0x0003_0000
//     v17 = sextend.i64 v3  ; v3 = 0x0003_0000
//     v18 = imul v16, v17
//     v19 = iconst.i64 16
//     v20 = sshr v18, v19  ; v19 = 16
//     v21 = ireduce.i32 v20
//     v22 = iconst.i32 1
//     return v22  ; v22 = 1
//
// block1:
//     v23 = iconst.i32 0
//     return v23  ; v23 = 0
// }

