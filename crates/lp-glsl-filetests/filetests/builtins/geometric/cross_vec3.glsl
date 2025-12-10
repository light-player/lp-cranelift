// test compile

// target riscv32.fixed32
vec3 main() {
    vec3 x = vec3(1.0, 0.0, 0.0);
    vec3 y = vec3(0.0, 1.0, 0.0);
    return cross(x, y);  // = (0.0, 0.0, 1.0)
}

// function u0:0(i32 sret) system_v {
// block0(v0: i32):
//     v19 = iconst.i32 0x0001_0000
//     v20 = iconst.i32 0
//     v21 = iconst.i32 0
//     v22 = iconst.i32 0
//     v23 = iconst.i32 0x0001_0000
//     v24 = iconst.i32 0
//     v25 = sextend.i64 v20  ; v20 = 0
//     v26 = sextend.i64 v24  ; v24 = 0
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = sextend.i64 v21  ; v21 = 0
//     v32 = sextend.i64 v23  ; v23 = 0x0001_0000
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = isub v30, v36
//     v38 = sextend.i64 v21  ; v21 = 0
//     v39 = sextend.i64 v22  ; v22 = 0
//     v40 = imul v38, v39
//     v41 = iconst.i64 16
//     v42 = sshr v40, v41  ; v41 = 16
//     v43 = ireduce.i32 v42
//     v44 = sextend.i64 v19  ; v19 = 0x0001_0000
//     v45 = sextend.i64 v24  ; v24 = 0
//     v46 = imul v44, v45
//     v47 = iconst.i64 16
//     v48 = sshr v46, v47  ; v47 = 16
//     v49 = ireduce.i32 v48
//     v50 = isub v43, v49
//     v51 = sextend.i64 v19  ; v19 = 0x0001_0000
//     v52 = sextend.i64 v23  ; v23 = 0x0001_0000
//     v53 = imul v51, v52
//     v54 = iconst.i64 16
//     v55 = sshr v53, v54  ; v54 = 16
//     v56 = ireduce.i32 v55
//     v57 = sextend.i64 v20  ; v20 = 0
//     v58 = sextend.i64 v22  ; v22 = 0
//     v59 = imul v57, v58
//     v60 = iconst.i64 16
//     v61 = sshr v59, v60  ; v60 = 16
//     v62 = ireduce.i32 v61
//     v63 = isub v56, v62
//     store notrap aligned v37, v0
//     store notrap aligned v50, v0+4
//     store notrap aligned v63, v0+8
//     return
//
// block1:
//     v64 = iconst.i32 0
//     store notrap aligned v64, v0  ; v64 = 0
//     v65 = iconst.i32 0
//     store notrap aligned v65, v0+4  ; v65 = 0
//     v66 = iconst.i32 0
//     store notrap aligned v66, v0+8  ; v66 = 0
//     return
// }
