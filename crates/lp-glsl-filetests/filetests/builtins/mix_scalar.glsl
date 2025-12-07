// test compile
// test run

bool main() {
    float result = mix(0.0, 10.0, 0.5);  // Should return 5.0: 0*(1-0.5) + 10*0.5
    // Validate result is approximately 5.0
    return result > 4.99 && result < 5.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0x1.400000p3
//     v2 = f32const 0x1.000000p-1
//     v3 = f32const 0x1.000000p0
//     v4 = fsub v3, v2  ; v3 = 0x1.000000p0, v2 = 0x1.000000p-1
//     v5 = fmul v0, v4  ; v0 = 0.0
//     v6 = fmul v1, v2  ; v1 = 0x1.400000p3, v2 = 0x1.000000p-1
//     v7 = fadd v5, v6
//     v8 = f32const 0x1.3f5c28p2
//     v9 = fcmp gt v7, v8  ; v8 = 0x1.3f5c28p2
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v9, v10, v11  ; v10 = 1, v11 = 0
//     v13 = f32const 0x1.40a3d8p2
//     v14 = fcmp lt v7, v13  ; v13 = 0x1.40a3d8p2
//     v15 = iconst.i8 1
//     v16 = iconst.i8 0
//     v17 = select v14, v15, v16  ; v15 = 1, v16 = 0
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
