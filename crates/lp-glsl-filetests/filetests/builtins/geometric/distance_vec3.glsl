// test compile

// target riscv32.fixed32
float main() {
    vec3 p0 = vec3(0.0, 0.0, 0.0);
    vec3 p1 = vec3(1.0, 2.0, 2.0);
    return distance(p0, p1);  // sqrt(1 + 4 + 4) = 3.0
}

// function u0:0() -> i32 system_v {
// block0:
//     v16 = iconst.i32 0
//     v17 = iconst.i32 0
//     v18 = iconst.i32 0
//     v19 = iconst.i32 0x0001_0000
//     v20 = iconst.i32 0x0002_0000
//     v21 = iconst.i32 0x0002_0000
//     v22 = isub v16, v19  ; v16 = 0, v19 = 0x0001_0000
//     v23 = isub v17, v20  ; v17 = 0, v20 = 0x0002_0000
//     v24 = isub v18, v21  ; v18 = 0, v21 = 0x0002_0000
//     v25 = sextend.i64 v22
//     v26 = sextend.i64 v22
//     v27 = imul v25, v26
//     v28 = iconst.i64 16
//     v29 = sshr v27, v28  ; v28 = 16
//     v30 = ireduce.i32 v29
//     v31 = sextend.i64 v23
//     v32 = sextend.i64 v23
//     v33 = imul v31, v32
//     v34 = iconst.i64 16
//     v35 = sshr v33, v34  ; v34 = 16
//     v36 = ireduce.i32 v35
//     v37 = iadd v30, v36
//     v38 = sextend.i64 v24
//     v39 = sextend.i64 v24
//     v40 = imul v38, v39
//     v41 = iconst.i64 16
//     v42 = sshr v40, v41  ; v41 = 16
//     v43 = ireduce.i32 v42
//     v44 = iadd v37, v43
//     v45 = iconst.i32 0
//     v46 = icmp eq v44, v45  ; v45 = 0
//     v47 = iconst.i32 8
//     v48 = sshr v44, v47  ; v47 = 8
//     v49 = sextend.i64 v44
//     v50 = sextend.i64 v48
//     v51 = ishl v49, v47  ; v47 = 8
//     v52 = sdiv v51, v50
//     v53 = iadd v50, v52
//     v54 = iconst.i64 1
//     v55 = sshr v53, v54  ; v54 = 1
//     v56 = sdiv v51, v55
//     v57 = iadd v55, v56
//     v58 = iconst.i64 1
//     v59 = sshr v57, v58  ; v58 = 1
//     v60 = sdiv v51, v59
//     v61 = iadd v59, v60
//     v62 = iconst.i64 1
//     v63 = sshr v61, v62  ; v62 = 1
//     v64 = sdiv v51, v63
//     v65 = iadd v63, v64
//     v66 = iconst.i64 1
//     v67 = sshr v65, v66  ; v66 = 1
//     v68 = sdiv v51, v67
//     v69 = iadd v67, v68
//     v70 = iconst.i64 1
//     v71 = sshr v69, v70  ; v70 = 1
//     v72 = ireduce.i32 v71
//     v73 = select v46, v45, v72  ; v45 = 0
//     return v73
//
// block1:
//     v74 = iconst.i32 0
//     return v74  ; v74 = 0
// }
