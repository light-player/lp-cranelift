// test compile

// target riscv32.fixed32
int main() {
    vec3 v = vec3(2.0, 3.0, 4.0);
    vec3 scaled = v * 2.0;  // (4.0, 6.0, 8.0)
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v9 = iconst.i32 0x0002_0000
//     v10 = iconst.i32 0x0003_0000
//     v11 = iconst.i32 0x0004_0000
//     v12 = iconst.i32 0x0002_0000
//     v13 = sextend.i64 v9  ; v9 = 0x0002_0000
//     v14 = sextend.i64 v12  ; v12 = 0x0002_0000
//     v15 = imul v13, v14
//     v16 = iconst.i64 16
//     v17 = sshr v15, v16  ; v16 = 16
//     v18 = ireduce.i32 v17
//     v19 = sextend.i64 v10  ; v10 = 0x0003_0000
//     v20 = sextend.i64 v12  ; v12 = 0x0002_0000
//     v21 = imul v19, v20
//     v22 = iconst.i64 16
//     v23 = sshr v21, v22  ; v22 = 16
//     v24 = ireduce.i32 v23
//     v25 = sextend.i64 v11  ; v11 = 0x0004_0000
//     v26 = sextend.i64 v12  ; v12 = 0x0002_0000
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v7 = iconst.i32 1
//     return v7  ; v7 = 1
//
// block1:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
// }

