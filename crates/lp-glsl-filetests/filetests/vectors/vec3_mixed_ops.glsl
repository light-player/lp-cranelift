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
//     v15 = iconst.i32 0x0001_0000
//     v16 = iconst.i32 0x0002_0000
//     v17 = iconst.i32 0x0003_0000
//     v18 = iconst.i32 0x0002_0000
//     v19 = iconst.i32 0x0002_0000
//     v20 = iconst.i32 0x0002_0000
//     v21 = iadd v15, v18  ; v15 = 0x0001_0000, v18 = 0x0002_0000
//     v22 = iadd v16, v19  ; v16 = 0x0002_0000, v19 = 0x0002_0000
//     v23 = iadd v17, v20  ; v17 = 0x0003_0000, v20 = 0x0002_0000
//     v24 = iconst.i32 0x0002_0000
//     v25 = sextend.i64 v21
//     v26 = sextend.i64 v24  ; v24 = 0x0002_0000
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = sextend.i64 v22
//     v32 = sextend.i64 v24  ; v24 = 0x0002_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = sextend.i64 v23
//     v38 = sextend.i64 v24  ; v24 = 0x0002_0000
//     v39 = imul v37, v38
//     v40 = iconst.i64 16
//     v41 = sshr v39, v40  ; v40 = 16
//     v42 = ireduce.i32 v41
//     v13 = iconst.i32 1
//     return v13  ; v13 = 1
//
// block1:
//     v14 = iconst.i32 0
//     return v14  ; v14 = 0
// }
