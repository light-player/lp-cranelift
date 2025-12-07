// test compile
// test run

bool main() {
    float a = sign(-5.0);  // -1.0
    float b = sign(0.0);   // 0.0
    float c = sign(5.0);   // 1.0
    // Validate sum: -1 + 0 + 1 = 0
    float sum = a + b + c;
    return sum > -0.01 && sum < 0.01;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = fneg v0  ; v0 = 0x1.400000p2
//     v2 = f32const 0.0
//     v3 = f32const 0x1.000000p0
//     v4 = f32const -0x1.000000p0
//     v5 = fcmp gt v1, v2  ; v2 = 0.0
//     v6 = fcmp lt v1, v2  ; v2 = 0.0
//     v7 = select v5, v3, v2  ; v3 = 0x1.000000p0, v2 = 0.0
//     v8 = select v6, v4, v7  ; v4 = -0x1.000000p0
//     v9 = f32const 0.0
//     v10 = f32const 0.0
//     v11 = f32const 0x1.000000p0
//     v12 = f32const -0x1.000000p0
//     v13 = fcmp gt v9, v10  ; v9 = 0.0, v10 = 0.0
//     v14 = fcmp lt v9, v10  ; v9 = 0.0, v10 = 0.0
//     v15 = select v13, v11, v10  ; v11 = 0x1.000000p0, v10 = 0.0
//     v16 = select v14, v12, v15  ; v12 = -0x1.000000p0
//     v17 = f32const 0x1.400000p2
//     v18 = f32const 0.0
//     v19 = f32const 0x1.000000p0
//     v20 = f32const -0x1.000000p0
//     v21 = fcmp gt v17, v18  ; v17 = 0x1.400000p2, v18 = 0.0
//     v22 = fcmp lt v17, v18  ; v17 = 0x1.400000p2, v18 = 0.0
//     v23 = select v21, v19, v18  ; v19 = 0x1.000000p0, v18 = 0.0
//     v24 = select v22, v20, v23  ; v20 = -0x1.000000p0
//     v25 = fadd v8, v16
//     v26 = fadd v25, v24
//     v27 = f32const 0x1.47ae14p-7
//     v28 = fneg v27  ; v27 = 0x1.47ae14p-7
//     v29 = fcmp gt v26, v28
//     v30 = iconst.i8 1
//     v31 = iconst.i8 0
//     v32 = select v29, v30, v31  ; v30 = 1, v31 = 0
//     v33 = f32const 0x1.47ae14p-7
//     v34 = fcmp lt v26, v33  ; v33 = 0x1.47ae14p-7
//     v35 = iconst.i8 1
//     v36 = iconst.i8 0
//     v37 = select v34, v35, v36  ; v35 = 1, v36 = 0
//     v38 = iconst.i8 0
//     v39 = iconst.i8 1
//     v40 = icmp ne v32, v38  ; v38 = 0
//     v41 = icmp ne v37, v38  ; v38 = 0
//     v42 = select v41, v39, v38  ; v39 = 1, v38 = 0
//     v43 = select v40, v42, v38  ; v38 = 0
//     return v43
//
// block1:
//     v44 = iconst.i8 0
//     return v44  ; v44 = 0
// }
// run: == true
