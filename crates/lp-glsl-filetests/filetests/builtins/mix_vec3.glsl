// test compile
// test run

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, vec3(0.5, 0.5, 0.5));  // (5.0, 10.0, 15.0)
    // Validate: x=5, y=10, z=15, sum=30
    float sum = result.x + result.y + result.z;
    return sum > 29.99 && sum < 30.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0.0
//     v2 = f32const 0.0
//     v3 = f32const 0x1.400000p3
//     v4 = f32const 0x1.400000p4
//     v5 = f32const 0x1.e00000p4
//     v6 = f32const 0x1.000000p-1
//     v7 = f32const 0x1.000000p-1
//     v8 = f32const 0x1.000000p-1
//     v9 = f32const 0x1.000000p0
//     v10 = fsub v9, v6  ; v9 = 0x1.000000p0, v6 = 0x1.000000p-1
//     v11 = fmul v0, v10  ; v0 = 0.0
//     v12 = fmul v3, v6  ; v3 = 0x1.400000p3, v6 = 0x1.000000p-1
//     v13 = fadd v11, v12
//     v14 = f32const 0x1.000000p0
//     v15 = fsub v14, v7  ; v14 = 0x1.000000p0, v7 = 0x1.000000p-1
//     v16 = fmul v1, v15  ; v1 = 0.0
//     v17 = fmul v4, v7  ; v4 = 0x1.400000p4, v7 = 0x1.000000p-1
//     v18 = fadd v16, v17
//     v19 = f32const 0x1.000000p0
//     v20 = fsub v19, v8  ; v19 = 0x1.000000p0, v8 = 0x1.000000p-1
//     v21 = fmul v2, v20  ; v2 = 0.0
//     v22 = fmul v5, v8  ; v5 = 0x1.e00000p4, v8 = 0x1.000000p-1
//     v23 = fadd v21, v22
//     v24 = fadd v13, v18
//     v25 = fadd v24, v23
//     v26 = f32const 0x1.dfd70ap4
//     v27 = fcmp gt v25, v26  ; v26 = 0x1.dfd70ap4
//     v28 = iconst.i8 1
//     v29 = iconst.i8 0
//     v30 = select v27, v28, v29  ; v28 = 1, v29 = 0
//     v31 = f32const 0x1.e028f6p4
//     v32 = fcmp lt v25, v31  ; v31 = 0x1.e028f6p4
//     v33 = iconst.i8 1
//     v34 = iconst.i8 0
//     v35 = select v32, v33, v34  ; v33 = 1, v34 = 0
//     v36 = iconst.i8 0
//     v37 = iconst.i8 1
//     v38 = icmp ne v30, v36  ; v36 = 0
//     v39 = icmp ne v35, v36  ; v36 = 0
//     v40 = select v39, v37, v36  ; v37 = 1, v36 = 0
//     v41 = select v38, v40, v36  ; v36 = 0
//     return v41
//
// block1:
//     v42 = iconst.i8 0
//     return v42  ; v42 = 0
// }
// run: == true
