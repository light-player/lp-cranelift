// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = mix(0.0, 10.0, 0.5);  // Should return 5.0: 0*(1-0.5) + 10*0.5
    // Validate result is approximately 5.0
    return result > 4.99 && result < 5.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v25 = iconst.i32 0
//     v26 = iconst.i32 0x000a_0000
//     v27 = iconst.i32 0x8000
//     v28 = iconst.i32 0x0001_0000
//     v29 = isub v28, v27  ; v28 = 0x0001_0000, v27 = 0x8000
//     v30 = sextend.i64 v25  ; v25 = 0
//     v31 = sextend.i64 v29
//     v32 = imul v30, v31
//     v33 = iconst.i64 16
//     v34 = sshr v32, v33  ; v33 = 16
//     v35 = ireduce.i32 v34
//     v36 = sextend.i64 v26  ; v26 = 0x000a_0000
//     v37 = sextend.i64 v27  ; v27 = 0x8000
//     v38 = imul v36, v37
//     v39 = iconst.i64 16
//     v40 = sshr v38, v39  ; v39 = 16
//     v41 = ireduce.i32 v40
//     v42 = iadd v35, v41
//     v43 = iconst.i32 0x0004_fd71
//     v44 = icmp sgt v42, v43  ; v43 = 0x0004_fd71
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v44, v10, v11  ; v10 = 1, v11 = 0
//     v45 = iconst.i32 0x0005_028f
//     v46 = icmp slt v42, v45  ; v45 = 0x0005_028f
//     v15 = iconst.i8 1
//     v16 = iconst.i8 0
//     v17 = select v46, v15, v16  ; v15 = 1, v16 = 0
//     v18 = iconst.i8 0
//     v19 = iconst.i8 1
//     v20 = icmp ne v12, v18  ; v18 = 0
//     v21 = icmp ne v17, v18  ; v18 = 0
//     v22 = select v21, v19, v18  ; v19 = 1, v18 = 0
//     v23 = select v20, v22, v18  ; v18 = 0
//     return v23
//
// block1:
//     v24 = iconst.i8 0
//     return v24  ; v24 = 0
// }
// run: == true
