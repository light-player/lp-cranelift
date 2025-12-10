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
//     v33 = iconst.i32 0
//     v34 = iconst.i32 0x000a_0000
//     v35 = iconst.i32 0x0005_0000
//     v36 = iconst.i32 0
//     v37 = iconst.i32 0x0001_0000
//     v38 = iconst.i32 0x0002_0000
//     v39 = iconst.i32 0x0003_0000
//     v40 = isub v35, v33  ; v35 = 0x0005_0000, v33 = 0
//     v41 = isub v34, v33  ; v34 = 0x000a_0000, v33 = 0
//     v42 = sextend.i64 v40
//     v43 = iconst.i64 16
//     v44 = ishl v42, v43  ; v43 = 16
//     v45 = sextend.i64 v41
//     v46 = sdiv v44, v45
//     v47 = ireduce.i32 v46
//     v48 = icmp sge v47, v36  ; v36 = 0
//     v49 = select v48, v47, v36  ; v36 = 0
//     v50 = icmp sle v49, v37  ; v37 = 0x0001_0000
//     v51 = select v50, v49, v37  ; v37 = 0x0001_0000
//     v52 = sextend.i64 v51
//     v53 = sextend.i64 v51
//     v54 = imul v52, v53
//     v55 = iconst.i64 16
//     v56 = sshr v54, v55  ; v55 = 16
//     v57 = ireduce.i32 v56
//     v58 = sextend.i64 v38  ; v38 = 0x0002_0000
//     v59 = sextend.i64 v51
//     v60 = imul v58, v59
//     v61 = iconst.i64 16
//     v62 = sshr v60, v61  ; v61 = 16
//     v63 = ireduce.i32 v62
//     v64 = isub v39, v63  ; v39 = 0x0003_0000
//     v65 = sextend.i64 v57
//     v66 = sextend.i64 v64
//     v67 = imul v65, v66
//     v68 = iconst.i64 16
//     v69 = sshr v67, v68  ; v68 = 16
//     v70 = ireduce.i32 v69
//     v71 = iconst.i32 0x7d71
//     v72 = icmp sgt v70, v71  ; v71 = 0x7d71
//     v18 = iconst.i8 1
//     v19 = iconst.i8 0
//     v20 = select v72, v18, v19  ; v18 = 1, v19 = 0
//     v73 = iconst.i32 0x828f
//     v74 = icmp slt v70, v73  ; v73 = 0x828f
//     v23 = iconst.i8 1
//     v24 = iconst.i8 0
//     v25 = select v74, v23, v24  ; v23 = 1, v24 = 0
//     v26 = iconst.i8 0
//     v27 = iconst.i8 1
//     v28 = icmp ne v20, v26  ; v26 = 0
//     v29 = icmp ne v25, v26  ; v26 = 0
//     v30 = select v29, v27, v26  ; v27 = 1, v26 = 0
//     v31 = select v28, v30, v26  ; v26 = 0
//     return v31
//
// block1:
//     v32 = iconst.i8 0
//     return v32  ; v32 = 0
// }
// run: == true
