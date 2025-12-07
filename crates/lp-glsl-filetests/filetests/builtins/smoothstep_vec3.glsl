// test compile
// test run

bool main() {
    vec3 edge0 = vec3(0.0, 0.0, 0.0);
    vec3 edge1 = vec3(10.0, 10.0, 10.0);
    vec3 x = vec3(5.0, 2.5, 7.5);
    vec3 result = smoothstep(edge0, edge1, x);
    // Expected: smoothstep(0,10,5)≈0.5, smoothstep(0,10,2.5)≈0.15625, smoothstep(0,10,7.5)≈0.84375
    // Sum ≈ 1.5
    float sum = result.x + result.y + result.z;
    return sum > 1.45 && sum < 1.55;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0.0
//     v2 = f32const 0.0
//     v3 = f32const 0x1.400000p3
//     v4 = f32const 0x1.400000p3
//     v5 = f32const 0x1.400000p3
//     v6 = f32const 0x1.400000p2
//     v7 = f32const 0x1.400000p1
//     v8 = f32const 0x1.e00000p2
//     v9 = f32const 0.0
//     v10 = f32const 0x1.000000p0
//     v11 = f32const 0x1.000000p1
//     v12 = f32const 0x1.800000p1
//     v13 = fsub v6, v0  ; v6 = 0x1.400000p2, v0 = 0.0
//     v14 = fsub v3, v0  ; v3 = 0x1.400000p3, v0 = 0.0
//     v15 = fdiv v13, v14
//     v16 = fmax v15, v9  ; v9 = 0.0
//     v17 = fmin v16, v10  ; v10 = 0x1.000000p0
//     v18 = fmul v17, v17
//     v19 = fmul v11, v17  ; v11 = 0x1.000000p1
//     v20 = fsub v12, v19  ; v12 = 0x1.800000p1
//     v21 = fmul v18, v20
//     v22 = fsub v7, v1  ; v7 = 0x1.400000p1, v1 = 0.0
//     v23 = fsub v4, v1  ; v4 = 0x1.400000p3, v1 = 0.0
//     v24 = fdiv v22, v23
//     v25 = fmax v24, v9  ; v9 = 0.0
//     v26 = fmin v25, v10  ; v10 = 0x1.000000p0
//     v27 = fmul v26, v26
//     v28 = fmul v11, v26  ; v11 = 0x1.000000p1
//     v29 = fsub v12, v28  ; v12 = 0x1.800000p1
//     v30 = fmul v27, v29
//     v31 = fsub v8, v2  ; v8 = 0x1.e00000p2, v2 = 0.0
//     v32 = fsub v5, v2  ; v5 = 0x1.400000p3, v2 = 0.0
//     v33 = fdiv v31, v32
//     v34 = fmax v33, v9  ; v9 = 0.0
//     v35 = fmin v34, v10  ; v10 = 0x1.000000p0
//     v36 = fmul v35, v35
//     v37 = fmul v11, v35  ; v11 = 0x1.000000p1
//     v38 = fsub v12, v37  ; v12 = 0x1.800000p1
//     v39 = fmul v36, v38
//     v40 = fadd v21, v30
//     v41 = fadd v40, v39
//     v42 = f32const 0x1.733334p0
//     v43 = fcmp gt v41, v42  ; v42 = 0x1.733334p0
//     v44 = iconst.i8 1
//     v45 = iconst.i8 0
//     v46 = select v43, v44, v45  ; v44 = 1, v45 = 0
//     v47 = f32const 0x1.8cccccp0
//     v48 = fcmp lt v41, v47  ; v47 = 0x1.8cccccp0
//     v49 = iconst.i8 1
//     v50 = iconst.i8 0
//     v51 = select v48, v49, v50  ; v49 = 1, v50 = 0
//     v52 = iconst.i8 0
//     v53 = iconst.i8 1
//     v54 = icmp ne v46, v52  ; v52 = 0
//     v55 = icmp ne v51, v52  ; v52 = 0
//     v56 = select v55, v53, v52  ; v53 = 1, v52 = 0
//     v57 = select v54, v56, v52  ; v52 = 0
//     return v57
//
// block1:
//     v58 = iconst.i8 0
//     return v58  ; v58 = 0
// }
// run: == true
