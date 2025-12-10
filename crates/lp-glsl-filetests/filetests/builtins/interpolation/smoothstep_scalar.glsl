// test compile
// test run
// target riscv32.fixed32

bool main() {
    // smoothstep(0, 10, 5): t = 0.5, result = t²(3-2t) = 0.25*2 = 0.5
    float result = smoothstep(0.0, 10.0, 5.0);
    return result > 0.49 && result < 0.51;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0x000a_0000
//     v2 = iconst.i32 0x0005_0000
//     v3 = iconst.i32 0
//     v4 = iconst.i32 0x0001_0000
//     v5 = iconst.i32 0x0002_0000
//     v6 = iconst.i32 0x0003_0000
//     v7 = isub v2, v0  ; v2 = 0x0005_0000, v0 = 0
//     v8 = isub v1, v0  ; v1 = 0x000a_0000, v0 = 0
//     v9 = iconst.i32 0
//     v10 = icmp eq v8, v9  ; v9 = 0
//     v11 = iconst.i32 0x7fff_0000
//     v12 = iconst.i32 -2147483648
//     v13 = icmp eq v7, v9  ; v9 = 0
//     v14 = icmp slt v7, v9  ; v9 = 0
//     v15 = select v14, v12, v11  ; v12 = -2147483648, v11 = 0x7fff_0000
//     v16 = select v13, v9, v15  ; v9 = 0
//     v17 = iconst.i32 1
//     v18 = select v10, v17, v8  ; v17 = 1
//     v19 = sextend.i64 v7
//     v20 = iconst.i64 16
//     v21 = ishl v19, v20  ; v20 = 16
//     v22 = sextend.i64 v18
//     v23 = sdiv v21, v22
//     v24 = ireduce.i32 v23
//     v25 = select v10, v16, v24
//     v26 = icmp sgt v25, v3  ; v3 = 0
//     v27 = select v26, v25, v3  ; v3 = 0
//     v28 = icmp slt v27, v4  ; v4 = 0x0001_0000
//     v29 = select v28, v27, v4  ; v4 = 0x0001_0000
//     v30 = sextend.i64 v29
//     v31 = sextend.i64 v29
//     v32 = imul v30, v31
//     v33 = iconst.i64 16
//     v34 = sshr v32, v33  ; v33 = 16
//     v35 = ireduce.i32 v34
//     v36 = sextend.i64 v5  ; v5 = 0x0002_0000
//     v37 = sextend.i64 v29
//     v38 = imul v36, v37
//     v39 = iconst.i64 16
//     v40 = sshr v38, v39  ; v39 = 16
//     v41 = ireduce.i32 v40
//     v42 = isub v6, v41  ; v6 = 0x0003_0000
//     v43 = sextend.i64 v35
//     v44 = sextend.i64 v42
//     v45 = imul v43, v44
//     v46 = iconst.i64 16
//     v47 = sshr v45, v46  ; v46 = 16
//     v48 = ireduce.i32 v47
//     v49 = iconst.i32 0x7d71
//     v50 = icmp sgt v48, v49  ; v49 = 0x7d71
//     v51 = sextend.i32 v50
//     v52 = iconst.i8 1
//     v53 = iconst.i8 0
//     v54 = select v51, v52, v53  ; v52 = 1, v53 = 0
//     v55 = iconst.i32 0x828f
//     v56 = icmp slt v48, v55  ; v55 = 0x828f
//     v57 = sextend.i32 v56
//     v58 = iconst.i8 1
//     v59 = iconst.i8 0
//     v60 = select v57, v58, v59  ; v58 = 1, v59 = 0
//     v61 = iconst.i8 0
//     v62 = iconst.i8 1
//     v63 = icmp ne v54, v61  ; v61 = 0
//     v64 = icmp ne v60, v61  ; v61 = 0
//     v65 = select v64, v62, v61  ; v62 = 1, v61 = 0
//     v66 = select v63, v65, v61  ; v61 = 0
//     return v66
//
// block1:
//     v67 = iconst.i8 0
//     return v67  ; v67 = 0
// }
// run: == true
