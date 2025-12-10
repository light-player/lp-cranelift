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
//     v0 = iconst.i32 0x0003_c000
//     v1 = iconst.i32 0x0005_4000
//     v2 = iconst.i32 16
//     v3 = sshr v0, v2  ; v0 = 0x0003_c000, v2 = 16
//     v4 = ishl v3, v2  ; v2 = 16
//     v5 = isub v0, v4  ; v0 = 0x0003_c000
//     v6 = iconst.i32 16
//     v7 = sshr v1, v6  ; v1 = 0x0005_4000, v6 = 16
//     v8 = ishl v7, v6  ; v6 = 16
//     v9 = isub v1, v8  ; v1 = 0x0005_4000
//     v10 = iadd v5, v9
//     v11 = iconst.i32 0xfd71
//     v12 = icmp sgt v10, v11  ; v11 = 0xfd71
//     v13 = sextend.i32 v12
//     v14 = iconst.i8 1
//     v15 = iconst.i8 0
//     v16 = select v13, v14, v15  ; v14 = 1, v15 = 0
//     v17 = iconst.i32 0x0001_028f
//     v18 = icmp slt v10, v17  ; v17 = 0x0001_028f
//     v19 = sextend.i32 v18
//     v20 = iconst.i8 1
//     v21 = iconst.i8 0
//     v22 = select v19, v20, v21  ; v20 = 1, v21 = 0
//     v23 = iconst.i8 0
//     v24 = iconst.i8 1
//     v25 = icmp ne v16, v23  ; v23 = 0
//     v26 = icmp ne v22, v23  ; v23 = 0
//     v27 = select v26, v24, v23  ; v24 = 1, v23 = 0
//     v28 = select v25, v27, v23  ; v23 = 0
//     return v28
//
// block1:
//     v29 = iconst.i8 0
//     return v29  ; v29 = 0
// }
// run: == true
