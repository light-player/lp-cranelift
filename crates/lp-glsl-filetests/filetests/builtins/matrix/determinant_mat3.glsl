// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    mat3 m = mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
    return determinant(m);
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0001_0000
//     v1 = iconst.i32 0
//     v2 = iconst.i32 0
//     v3 = iconst.i32 0
//     v4 = iconst.i32 0x0001_0000
//     v5 = iconst.i32 0
//     v6 = iconst.i32 0
//     v7 = iconst.i32 0
//     v8 = iconst.i32 0x0001_0000
//     v9 = sextend.i64 v4  ; v4 = 0x0001_0000
//     v10 = sextend.i64 v8  ; v8 = 0x0001_0000
//     v11 = imul v9, v10
//     v12 = iconst.i64 16
//     v13 = sshr v11, v12  ; v12 = 16
//     v14 = ireduce.i32 v13
//     v15 = sextend.i64 v7  ; v7 = 0
//     v16 = sextend.i64 v5  ; v5 = 0
//     v17 = imul v15, v16
//     v18 = iconst.i64 16
//     v19 = sshr v17, v18  ; v18 = 16
//     v20 = ireduce.i32 v19
//     v21 = isub v14, v20
//     v22 = sextend.i64 v0  ; v0 = 0x0001_0000
//     v23 = sextend.i64 v21
//     v24 = imul v22, v23
//     v25 = iconst.i64 16
//     v26 = sshr v24, v25  ; v25 = 16
//     v27 = ireduce.i32 v26
//     v28 = sextend.i64 v1  ; v1 = 0
//     v29 = sextend.i64 v8  ; v8 = 0x0001_0000
//     v30 = imul v28, v29
//     v31 = iconst.i64 16
//     v32 = sshr v30, v31  ; v31 = 16
//     v33 = ireduce.i32 v32
//     v34 = sextend.i64 v7  ; v7 = 0
//     v35 = sextend.i64 v2  ; v2 = 0
//     v36 = imul v34, v35
//     v37 = iconst.i64 16
//     v38 = sshr v36, v37  ; v37 = 16
//     v39 = ireduce.i32 v38
//     v40 = isub v33, v39
//     v41 = sextend.i64 v3  ; v3 = 0
//     v42 = sextend.i64 v40
//     v43 = imul v41, v42
//     v44 = iconst.i64 16
//     v45 = sshr v43, v44  ; v44 = 16
//     v46 = ireduce.i32 v45
//     v47 = sextend.i64 v1  ; v1 = 0
//     v48 = sextend.i64 v5  ; v5 = 0
//     v49 = imul v47, v48
//     v50 = iconst.i64 16
//     v51 = sshr v49, v50  ; v50 = 16
//     v52 = ireduce.i32 v51
//     v53 = sextend.i64 v4  ; v4 = 0x0001_0000
//     v54 = sextend.i64 v2  ; v2 = 0
//     v55 = imul v53, v54
//     v56 = iconst.i64 16
//     v57 = sshr v55, v56  ; v56 = 16
//     v58 = ireduce.i32 v57
//     v59 = isub v52, v58
//     v60 = sextend.i64 v6  ; v6 = 0
//     v61 = sextend.i64 v59
//     v62 = imul v60, v61
//     v63 = iconst.i64 16
//     v64 = sshr v62, v63  ; v63 = 16
//     v65 = ireduce.i32 v64
//     v66 = isub v27, v46
//     v67 = iadd v66, v65
//     return v67
//
// block1:
//     v68 = iconst.i32 0
//     return v68  ; v68 = 0
// }
// run: ~= 1.0  // Identity matrix has determinant 1
