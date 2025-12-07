// test compile
// test run

bool main() {
    vec3 result = step(5.0, vec3(3.0, 5.0, 7.0));  // (0.0, 1.0, 1.0)
    // Validate: sum = 2.0
    float sum = result.x + result.y + result.z;
    return sum > 1.99 && sum < 2.01;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = f32const 0x1.800000p1
//     v2 = f32const 0x1.400000p2
//     v3 = f32const 0x1.c00000p2
//     v4 = f32const 0.0
//     v5 = f32const 0x1.000000p0
//     v6 = fcmp lt v1, v0  ; v1 = 0x1.800000p1, v0 = 0x1.400000p2
//     v7 = select v6, v4, v5  ; v4 = 0.0, v5 = 0x1.000000p0
//     v8 = fcmp lt v2, v0  ; v2 = 0x1.400000p2, v0 = 0x1.400000p2
//     v9 = select v8, v4, v5  ; v4 = 0.0, v5 = 0x1.000000p0
//     v10 = fcmp lt v3, v0  ; v3 = 0x1.c00000p2, v0 = 0x1.400000p2
//     v11 = select v10, v4, v5  ; v4 = 0.0, v5 = 0x1.000000p0
//     v12 = fadd v7, v9
//     v13 = fadd v12, v11
//     v14 = f32const 0x1.fd70a4p0
//     v15 = fcmp gt v13, v14  ; v14 = 0x1.fd70a4p0
//     v16 = iconst.i8 1
//     v17 = iconst.i8 0
//     v18 = select v15, v16, v17  ; v16 = 1, v17 = 0
//     v19 = f32const 0x1.0147aep1
//     v20 = fcmp lt v13, v19  ; v19 = 0x1.0147aep1
//     v21 = iconst.i8 1
//     v22 = iconst.i8 0
//     v23 = select v20, v21, v22  ; v21 = 1, v22 = 0
//     v24 = iconst.i8 0
//     v25 = iconst.i8 1
//     v26 = icmp ne v18, v24  ; v24 = 0
//     v27 = icmp ne v23, v24  ; v24 = 0
//     v28 = select v27, v25, v24  ; v25 = 1, v24 = 0
//     v29 = select v26, v28, v24  ; v24 = 0
//     return v29
//
// block1:
//     v30 = iconst.i8 0
//     return v30  ; v30 = 0
// }
// run: == true
