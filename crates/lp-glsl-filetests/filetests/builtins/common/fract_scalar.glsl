// test compile
// test run
// target riscv32.fixed32

bool main() {
    float result = fract(3.75);  // 3.75 - floor(3.75) = 3.75 - 3.0 = 0.75
    return result > 0.74 && result < 0.76;
}

// function u0:0() -> i8 system_v {
// block0:
//     v20 = iconst.i32 0x0003_c000
//     v21 = iconst.i64 16
//     v22 = sextend.i64 v20  ; v20 = 0x0003_c000
//     v23 = sshr v22, v21  ; v21 = 16
//     v24 = ishl v23, v21  ; v21 = 16
//     v25 = ireduce.i32 v24
//     v26 = isub v20, v25  ; v20 = 0x0003_c000
//     v27 = iconst.i32 0xbd71
//     v28 = icmp sgt v26, v27  ; v27 = 0xbd71
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v28, v5, v6  ; v5 = 1, v6 = 0
//     v29 = iconst.i32 0xc28f
//     v30 = icmp slt v26, v29  ; v29 = 0xc28f
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v30, v10, v11  ; v10 = 1, v11 = 0
//     v13 = iconst.i8 0
//     v14 = iconst.i8 1
//     v15 = icmp ne v7, v13  ; v13 = 0
//     v16 = icmp ne v12, v13  ; v13 = 0
//     v17 = select v16, v14, v13  ; v14 = 1, v13 = 0
//     v18 = select v15, v17, v13  ; v13 = 0
//     return v18
//
// block1:
//     v19 = iconst.i8 0
//     return v19  ; v19 = 0
// }
// run: == true
