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
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0x000a_0000
//     v2 = iconst.i32 0x8000
//     v3 = iconst.i32 0x0001_0000
//     v4 = isub v3, v2  ; v3 = 0x0001_0000, v2 = 0x8000
//     v5 = sextend.i64 v0  ; v0 = 0
//     v6 = sextend.i64 v4
//     v7 = imul v5, v6
//     v8 = iconst.i64 16
//     v9 = sshr v7, v8  ; v8 = 16
//     v10 = ireduce.i32 v9
//     v11 = sextend.i64 v1  ; v1 = 0x000a_0000
//     v12 = sextend.i64 v2  ; v2 = 0x8000
//     v13 = imul v11, v12
//     v14 = iconst.i64 16
//     v15 = sshr v13, v14  ; v14 = 16
//     v16 = ireduce.i32 v15
//     v17 = iadd v10, v16
//     v18 = iconst.i32 0x0004_fd71
//     v19 = icmp sgt v17, v18  ; v18 = 0x0004_fd71
//     v20 = sextend.i32 v19
//     v21 = iconst.i8 1
//     v22 = iconst.i8 0
//     v23 = select v20, v21, v22  ; v21 = 1, v22 = 0
//     v24 = iconst.i32 0x0005_028f
//     v25 = icmp slt v17, v24  ; v24 = 0x0005_028f
//     v26 = sextend.i32 v25
//     v27 = iconst.i8 1
//     v28 = iconst.i8 0
//     v29 = select v26, v27, v28  ; v27 = 1, v28 = 0
//     v30 = iconst.i8 0
//     v31 = iconst.i8 1
//     v32 = icmp ne v23, v30  ; v30 = 0
//     v33 = icmp ne v29, v30  ; v30 = 0
//     v34 = select v33, v31, v30  ; v31 = 1, v30 = 0
//     v35 = select v32, v34, v30  ; v30 = 0
//     return v35
//
// block1:
//     v36 = iconst.i8 0
//     return v36  ; v36 = 0
// }
// run: == true
