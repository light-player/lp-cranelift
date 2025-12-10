// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = mod(7.0, 3.0);  // 7 - 3*floor(7/3) = 7 - 3*2 = 1.0
    return result > 0.99 && result < 1.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v23 = iconst.i32 0x0007_0000
//     v24 = iconst.i32 0x0003_0000
//     v25 = sextend.i64 v23  ; v23 = 0x0007_0000
//     v26 = iconst.i64 16
//     v27 = ishl v25, v26  ; v26 = 16
//     v28 = sextend.i64 v24  ; v24 = 0x0003_0000
//     v29 = sdiv v27, v28
//     v30 = ireduce.i32 v29
//     v31 = iconst.i64 16
//     v32 = sextend.i64 v30
//     v33 = sshr v32, v31  ; v31 = 16
//     v34 = ishl v33, v31  ; v31 = 16
//     v35 = ireduce.i32 v34
//     v36 = sextend.i64 v24  ; v24 = 0x0003_0000
//     v37 = sextend.i64 v35
//     v38 = imul v36, v37
//     v39 = iconst.i64 16
//     v40 = sshr v38, v39  ; v39 = 16
//     v41 = ireduce.i32 v40
//     v42 = isub v23, v41  ; v23 = 0x0007_0000
//     v43 = iconst.i32 0xfd71
//     v44 = icmp sgt v42, v43  ; v43 = 0xfd71
//     v8 = iconst.i8 1
//     v9 = iconst.i8 0
//     v10 = select v44, v8, v9  ; v8 = 1, v9 = 0
//     v45 = iconst.i32 0x0001_028f
//     v46 = icmp slt v42, v45  ; v45 = 0x0001_028f
//     v13 = iconst.i8 1
//     v14 = iconst.i8 0
//     v15 = select v46, v13, v14  ; v13 = 1, v14 = 0
//     v16 = iconst.i8 0
//     v17 = iconst.i8 1
//     v18 = icmp ne v10, v16  ; v16 = 0
//     v19 = icmp ne v15, v16  ; v16 = 0
//     v20 = select v19, v17, v16  ; v17 = 1, v16 = 0
//     v21 = select v18, v20, v16  ; v16 = 0
//     return v21
//
// block1:
//     v22 = iconst.i8 0
//     return v22  ; v22 = 0
// }
// run: == true
