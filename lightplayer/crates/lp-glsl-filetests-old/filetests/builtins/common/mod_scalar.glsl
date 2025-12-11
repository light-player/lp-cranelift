// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = mod(7.0, 3.0);  // 7 - 3*floor(7/3) = 7 - 3*2 = 1.0
    return result > 0.99 && result < 1.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i32 0x0007_0000
//     v1 = iconst.i32 0x0003_0000
//     v2 = iconst.i32 0
//     v3 = icmp eq v1, v2  ; v1 = 0x0003_0000, v2 = 0
//     v4 = iconst.i32 0x7fff_0000
//     v5 = iconst.i32 -2147483648
//     v6 = icmp eq v0, v2  ; v0 = 0x0007_0000, v2 = 0
//     v7 = icmp slt v0, v2  ; v0 = 0x0007_0000, v2 = 0
//     v8 = select v7, v5, v4  ; v5 = -2147483648, v4 = 0x7fff_0000
//     v9 = select v6, v2, v8  ; v2 = 0
//     v10 = iconst.i32 1
//     v11 = select v3, v10, v1  ; v10 = 1, v1 = 0x0003_0000
//     v12 = sextend.i64 v0  ; v0 = 0x0007_0000
//     v13 = iconst.i64 16
//     v14 = ishl v12, v13  ; v13 = 16
//     v15 = sextend.i64 v11
//     v16 = sdiv v14, v15
//     v17 = ireduce.i32 v16
//     v18 = select v3, v9, v17
//     v19 = iconst.i32 16
//     v20 = sshr v18, v19  ; v19 = 16
//     v21 = ishl v20, v19  ; v19 = 16
//     v22 = sextend.i64 v1  ; v1 = 0x0003_0000
//     v23 = sextend.i64 v21
//     v24 = imul v22, v23
//     v25 = iconst.i64 16
//     v26 = sshr v24, v25  ; v25 = 16
//     v27 = ireduce.i32 v26
//     v28 = isub v0, v27  ; v0 = 0x0007_0000
//     v29 = iconst.i32 0xfd71
//     v30 = icmp sgt v28, v29  ; v29 = 0xfd71
//     v31 = sextend.i32 v30
//     v32 = iconst.i8 1
//     v33 = iconst.i8 0
//     v34 = select v31, v32, v33  ; v32 = 1, v33 = 0
//     v35 = iconst.i32 0x0001_028f
//     v36 = icmp slt v28, v35  ; v35 = 0x0001_028f
//     v37 = sextend.i32 v36
//     v38 = iconst.i8 1
//     v39 = iconst.i8 0
//     v40 = select v37, v38, v39  ; v38 = 1, v39 = 0
//     v41 = iconst.i8 0
//     v42 = iconst.i8 1
//     v43 = icmp ne v34, v41  ; v41 = 0
//     v44 = icmp ne v40, v41  ; v41 = 0
//     v45 = select v44, v42, v41  ; v42 = 1, v41 = 0
//     v46 = select v43, v45, v41  ; v41 = 0
//     return v46
//
// block1:
//     v47 = iconst.i8 0
//     return v47  ; v47 = 0
// }
// run: == true
