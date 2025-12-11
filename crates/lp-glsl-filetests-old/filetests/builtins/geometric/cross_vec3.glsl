// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 x = vec3(1.0, 0.0, 0.0);
    vec3 y = vec3(0.0, 1.0, 0.0);
    return cross(x, y);  // = (0.0, 0.0, 1.0)
}

// function u0:0(i32) system_v {
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0
//     v4 = iconst.i32 0
//     v5 = iconst.i32 0x0001_0000
//     v6 = iconst.i32 0
//     v7 = sextend.i64 v2  ; v2 = 0
//     v8 = sextend.i64 v6  ; v6 = 0
//     v9 = imul v7, v8
//     v10 = iconst.i64 16
//     v11 = sshr v9, v10  ; v10 = 16
//     v12 = ireduce.i32 v11
//     v13 = sextend.i64 v3  ; v3 = 0
//     v14 = sextend.i64 v5  ; v5 = 0x0001_0000
//     v15 = imul v13, v14
//     v16 = iconst.i64 16
//     v17 = sshr v15, v16  ; v16 = 16
//     v18 = ireduce.i32 v17
//     v19 = isub v12, v18
//     v20 = sextend.i64 v3  ; v3 = 0
//     v21 = sextend.i64 v4  ; v4 = 0
//     v22 = imul v20, v21
//     v23 = iconst.i64 16
//     v24 = sshr v22, v23  ; v23 = 16
//     v25 = ireduce.i32 v24
//     v26 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v27 = sextend.i64 v6  ; v6 = 0
//     v28 = imul v26, v27
//     v29 = iconst.i64 16
//     v30 = sshr v28, v29  ; v29 = 16
//     v31 = ireduce.i32 v30
//     v32 = isub v25, v31
//     v33 = sextend.i64 v1  ; v1 = 0x0001_0000
//     v34 = sextend.i64 v5  ; v5 = 0x0001_0000
//     v35 = imul v33, v34
//     v36 = iconst.i64 16
//     v37 = sshr v35, v36  ; v36 = 16
//     v38 = ireduce.i32 v37
//     v39 = sextend.i64 v2  ; v2 = 0
//     v40 = sextend.i64 v4  ; v4 = 0
//     v41 = imul v39, v40
//     v42 = iconst.i64 16
//     v43 = sshr v41, v42  ; v42 = 16
//     v44 = ireduce.i32 v43
//     v45 = isub v38, v44
//     store notrap aligned v19, v0
//     store notrap aligned v32, v0+4
//     store notrap aligned v45, v0+8
//     return
//
// block1:
//     v46 = iconst.i32 0
//     store notrap aligned v46, v0  ; v46 = 0
//     v47 = iconst.i32 0
//     store notrap aligned v47, v0+4  ; v47 = 0
//     v48 = iconst.i32 0
//     store notrap aligned v48, v0+8  ; v48 = 0
//     return
// }
