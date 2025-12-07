// test compile
// test run

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, 0.25);  // 0*0.75 + vec3(10,20,30)*0.25 = (2.5, 5.0, 7.5)
    // Validate: sum = 2.5 + 5.0 + 7.5 = 15.0
    float sum = result.x + result.y + result.z;
    return sum > 14.99 && sum < 15.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0.0
//     v2 = f32const 0.0
//     v3 = f32const 0x1.400000p3
//     v4 = f32const 0x1.400000p4
//     v5 = f32const 0x1.e00000p4
//     v6 = f32const 0x1.000000p-2
//     v7 = f32const 0x1.000000p0
//     v8 = fsub v7, v6  ; v7 = 0x1.000000p0, v6 = 0x1.000000p-2
//     v9 = fmul v0, v8  ; v0 = 0.0
//     v10 = fmul v3, v6  ; v3 = 0x1.400000p3, v6 = 0x1.000000p-2
//     v11 = fadd v9, v10
//     v12 = fmul v1, v8  ; v1 = 0.0
//     v13 = fmul v4, v6  ; v4 = 0x1.400000p4, v6 = 0x1.000000p-2
//     v14 = fadd v12, v13
//     v15 = fmul v2, v8  ; v2 = 0.0
//     v16 = fmul v5, v6  ; v5 = 0x1.e00000p4, v6 = 0x1.000000p-2
//     v17 = fadd v15, v16
//     v18 = fadd v11, v14
//     v19 = fadd v18, v17
//     v20 = f32const 0x1.dfae14p3
//     v21 = fcmp gt v19, v20  ; v20 = 0x1.dfae14p3
//     v22 = iconst.i8 1
//     v23 = iconst.i8 0
//     v24 = select v21, v22, v23  ; v22 = 1, v23 = 0
//     v25 = f32const 0x1.e051ecp3
//     v26 = fcmp lt v19, v25  ; v25 = 0x1.e051ecp3
//     v27 = iconst.i8 1
//     v28 = iconst.i8 0
//     v29 = select v26, v27, v28  ; v27 = 1, v28 = 0
//     v30 = iconst.i8 0
//     v31 = iconst.i8 1
//     v32 = icmp ne v24, v30  ; v30 = 0
//     v33 = icmp ne v29, v30  ; v30 = 0
//     v34 = select v33, v31, v30  ; v31 = 1, v30 = 0
//     v35 = select v32, v34, v30  ; v30 = 0
//     return v35
//
// block1:
//     v36 = iconst.i8 0
//     return v36  ; v36 = 0
// }
// run: == true
