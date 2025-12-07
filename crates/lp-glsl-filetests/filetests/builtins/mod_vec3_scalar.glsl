// test compile
// test run

bool main() {
    vec3 result = mod(vec3(7.0, 8.0, 9.0), 3.0);  // (1.0, 2.0, 0.0)
    // Validate: sum = 1 + 2 + 0 = 3.0
    float sum = result.x + result.y + result.z;
    return sum > 2.99 && sum < 3.01;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = f32const 0x1.c00000p2
//     v1 = f32const 0x1.000000p3
//     v2 = f32const 0x1.200000p3
//     v3 = f32const 0x1.800000p1
//     v4 = fdiv v0, v3  ; v0 = 0x1.c00000p2, v3 = 0x1.800000p1
//     v5 = floor v4
//     v6 = fmul v3, v5  ; v3 = 0x1.800000p1
//     v7 = fsub v0, v6  ; v0 = 0x1.c00000p2
//     v8 = fdiv v1, v3  ; v1 = 0x1.000000p3, v3 = 0x1.800000p1
//     v9 = floor v8
//     v10 = fmul v3, v9  ; v3 = 0x1.800000p1
//     v11 = fsub v1, v10  ; v1 = 0x1.000000p3
//     v12 = fdiv v2, v3  ; v2 = 0x1.200000p3, v3 = 0x1.800000p1
//     v13 = floor v12
//     v14 = fmul v3, v13  ; v3 = 0x1.800000p1
//     v15 = fsub v2, v14  ; v2 = 0x1.200000p3
//     v16 = fadd v7, v11
//     v17 = fadd v16, v15
//     v18 = f32const 0x1.7eb852p1
//     v19 = fcmp gt v17, v18  ; v18 = 0x1.7eb852p1
//     v20 = iconst.i8 1
//     v21 = iconst.i8 0
//     v22 = select v19, v20, v21  ; v20 = 1, v21 = 0
//     v23 = f32const 0x1.8147aep1
//     v24 = fcmp lt v17, v23  ; v23 = 0x1.8147aep1
//     v25 = iconst.i8 1
//     v26 = iconst.i8 0
//     v27 = select v24, v25, v26  ; v25 = 1, v26 = 0
//     v28 = iconst.i8 0
//     v29 = iconst.i8 1
//     v30 = icmp ne v22, v28  ; v28 = 0
//     v31 = icmp ne v27, v28  ; v28 = 0
//     v32 = select v31, v29, v28  ; v29 = 1, v28 = 0
//     v33 = select v30, v32, v28  ; v28 = 0
//     return v33
//
// block1:
//     v34 = iconst.i8 0
//     return v34  ; v34 = 0
// }
// run: == true
