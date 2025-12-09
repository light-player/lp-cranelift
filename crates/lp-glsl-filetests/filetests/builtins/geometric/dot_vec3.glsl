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
//     v12 = iconst.i32 0x0001_0000
//     v13 = iconst.i32 0x0002_0000
//     v14 = iconst.i32 0x0003_0000
//     v15 = iconst.i32 0x0004_0000
//     v16 = iconst.i32 0x0005_0000
//     v17 = iconst.i32 0x0006_0000
//     v18 = sextend.i64 v12  ; v12 = 0x0001_0000
//     v19 = sextend.i64 v15  ; v15 = 0x0004_0000
//     v20 = imul v18, v19
//     v21 = iconst.i64 16
//     v22 = sshr v20, v21  ; v21 = 16
//     v23 = ireduce.i32 v22
//     v24 = sextend.i64 v13  ; v13 = 0x0002_0000
//     v25 = sextend.i64 v16  ; v16 = 0x0005_0000
//     v26 = imul v24, v25
//     v27 = iconst.i64 16
//     v28 = sshr v26, v27  ; v27 = 16
//     v29 = ireduce.i32 v28
//     v30 = iadd v23, v29
//     v31 = sextend.i64 v14  ; v14 = 0x0003_0000
//     v32 = sextend.i64 v17  ; v17 = 0x0006_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = iadd v30, v36
//     return v37
//
// block1:
//     v38 = iconst.i32 0
//     return v38  ; v38 = 0
// }
// run: ~= 32
