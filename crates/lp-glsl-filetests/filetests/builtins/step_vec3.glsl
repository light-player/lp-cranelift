// test compile
// test run

bool main() {
    vec3 edge = vec3(5.0, 5.0, 5.0);
    vec3 x = vec3(3.0, 5.0, 7.0);
    vec3 result = step(edge, x);  // (0.0, 1.0, 1.0) because 3<5, 5>=5, 7>5
    // Validate: sum = 0 + 1 + 1 = 2.0
    float sum = result.x + result.y + result.z;
    return sum > 1.99 && sum < 2.01;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = f32const 0x1.400000p2
//     v2 = f32const 0x1.400000p2
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.400000p2
//     v5 = f32const 0x1.c00000p2
//     v6 = f32const 0.0
//     v7 = f32const 0x1.000000p0
//     v8 = fcmp lt v3, v0  ; v3 = 0x1.800000p1, v0 = 0x1.400000p2
//     v9 = select v8, v6, v7  ; v6 = 0.0, v7 = 0x1.000000p0
//     v10 = fcmp lt v4, v1  ; v4 = 0x1.400000p2, v1 = 0x1.400000p2
//     v11 = select v10, v6, v7  ; v6 = 0.0, v7 = 0x1.000000p0
//     v12 = fcmp lt v5, v2  ; v5 = 0x1.c00000p2, v2 = 0x1.400000p2
//     v13 = select v12, v6, v7  ; v6 = 0.0, v7 = 0x1.000000p0
//     v14 = fadd v9, v11
//     v15 = fadd v14, v13
//     v16 = f32const 0x1.fd70a4p0
//     v17 = fcmp gt v15, v16  ; v16 = 0x1.fd70a4p0
//     v18 = iconst.i8 1
//     v19 = iconst.i8 0
//     v20 = select v17, v18, v19  ; v18 = 1, v19 = 0
//     v21 = f32const 0x1.0147aep1
//     v22 = fcmp lt v15, v21  ; v21 = 0x1.0147aep1
//     v23 = iconst.i8 1
//     v24 = iconst.i8 0
//     v25 = select v22, v23, v24  ; v23 = 1, v24 = 0
//     v26 = iconst.i8 0
//     v27 = iconst.i8 1
//     v28 = icmp ne v20, v26  ; v26 = 0
//     v29 = icmp ne v25, v26  ; v26 = 0
//     v30 = select v29, v27, v26  ; v27 = 1, v26 = 0
//     v31 = select v28, v30, v26  ; v26 = 0
//     return v31
//
// block1:
//     v32 = iconst.i8 0
//     return v32  ; v32 = 0
// }
// run: == true
