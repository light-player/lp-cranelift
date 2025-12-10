// test compile
// test run
// target riscv32.fixed32

bool main() {
    vec2 result = fract(vec2(3.75, 5.25));  // (0.75, 0.25)
    // Validate: sum = 0.75 + 0.25 = 1.0
    float sum = result.x + result.y;
    return sum > 0.99 && sum < 1.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v24 = iconst.i32 0x0003_c000
//     v25 = iconst.i32 0x0005_4000
//     v26 = iconst.i64 16
//     v27 = sextend.i64 v24  ; v24 = 0x0003_c000
//     v28 = sshr v27, v26  ; v26 = 16
//     v29 = ishl v28, v26  ; v26 = 16
//     v30 = ireduce.i32 v29
//     v31 = isub v24, v30  ; v24 = 0x0003_c000
//     v32 = iconst.i64 16
//     v33 = sextend.i64 v25  ; v25 = 0x0005_4000
//     v34 = sshr v33, v32  ; v32 = 16
//     v35 = ishl v34, v32  ; v32 = 16
//     v36 = ireduce.i32 v35
//     v37 = isub v25, v36  ; v25 = 0x0005_4000
//     v38 = iadd v31, v37
//     v39 = iconst.i32 0xfd71
//     v40 = icmp sgt v38, v39  ; v39 = 0xfd71
//     v9 = iconst.i8 1
//     v10 = iconst.i8 0
//     v11 = select v40, v9, v10  ; v9 = 1, v10 = 0
//     v41 = iconst.i32 0x0001_028f
//     v42 = icmp slt v38, v41  ; v41 = 0x0001_028f
//     v14 = iconst.i8 1
//     v15 = iconst.i8 0
//     v16 = select v42, v14, v15  ; v14 = 1, v15 = 0
//     v17 = iconst.i8 0
//     v18 = iconst.i8 1
//     v19 = icmp ne v11, v17  ; v17 = 0
//     v20 = icmp ne v16, v17  ; v17 = 0
//     v21 = select v20, v18, v17  ; v18 = 1, v17 = 0
//     v22 = select v19, v21, v17  ; v17 = 0
//     return v22
//
// block1:
//     v23 = iconst.i8 0
//     return v23  ; v23 = 0
// }
// run: == true
