// test compile
// test run

bool main() {
    vec3 result = smoothstep(0.0, 10.0, vec3(0.0, 5.0, 10.0));
    // Expected: smoothstep(0,10,0)=0, smoothstep(0,10,5)=0.5, smoothstep(0,10,10)=1.0
    // Sum = 1.5
    float sum = result.x + result.y + result.z;
    return sum > 1.49 && sum < 1.51;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0x1.400000p3
//     v2 = f32const 0.0
//     v3 = f32const 0x1.400000p2
//     v4 = f32const 0x1.400000p3
//     v5 = f32const 0.0
//     v6 = f32const 0x1.000000p0
//     v7 = f32const 0x1.000000p1
//     v8 = f32const 0x1.800000p1
//     v9 = fsub v2, v0  ; v2 = 0.0, v0 = 0.0
//     v10 = fsub v1, v0  ; v1 = 0x1.400000p3, v0 = 0.0
//     v11 = fdiv v9, v10
//     v12 = fmax v11, v5  ; v5 = 0.0
//     v13 = fmin v12, v6  ; v6 = 0x1.000000p0
//     v14 = fmul v13, v13
//     v15 = fmul v7, v13  ; v7 = 0x1.000000p1
//     v16 = fsub v8, v15  ; v8 = 0x1.800000p1
//     v17 = fmul v14, v16
//     v18 = fsub v3, v0  ; v3 = 0x1.400000p2, v0 = 0.0
//     v19 = fsub v1, v0  ; v1 = 0x1.400000p3, v0 = 0.0
//     v20 = fdiv v18, v19
//     v21 = fmax v20, v5  ; v5 = 0.0
//     v22 = fmin v21, v6  ; v6 = 0x1.000000p0
//     v23 = fmul v22, v22
//     v24 = fmul v7, v22  ; v7 = 0x1.000000p1
//     v25 = fsub v8, v24  ; v8 = 0x1.800000p1
//     v26 = fmul v23, v25
//     v27 = fsub v4, v0  ; v4 = 0x1.400000p3, v0 = 0.0
//     v28 = fsub v1, v0  ; v1 = 0x1.400000p3, v0 = 0.0
//     v29 = fdiv v27, v28
//     v30 = fmax v29, v5  ; v5 = 0.0
//     v31 = fmin v30, v6  ; v6 = 0x1.000000p0
//     v32 = fmul v31, v31
//     v33 = fmul v7, v31  ; v7 = 0x1.000000p1
//     v34 = fsub v8, v33  ; v8 = 0x1.800000p1
//     v35 = fmul v32, v34
//     v36 = fadd v17, v26
//     v37 = fadd v36, v35
//     v38 = f32const 0x1.7d70a4p0
//     v39 = fcmp gt v37, v38  ; v38 = 0x1.7d70a4p0
//     v40 = iconst.i8 1
//     v41 = iconst.i8 0
//     v42 = select v39, v40, v41  ; v40 = 1, v41 = 0
//     v43 = f32const 0x1.828f5cp0
//     v44 = fcmp lt v37, v43  ; v43 = 0x1.828f5cp0
//     v45 = iconst.i8 1
//     v46 = iconst.i8 0
//     v47 = select v44, v45, v46  ; v45 = 1, v46 = 0
//     v48 = iconst.i8 0
//     v49 = iconst.i8 1
//     v50 = icmp ne v42, v48  ; v48 = 0
//     v51 = icmp ne v47, v48  ; v48 = 0
//     v52 = select v51, v49, v48  ; v49 = 1, v48 = 0
//     v53 = select v50, v52, v48  ; v48 = 0
//     return v53
//
// block1:
//     v54 = iconst.i8 0
//     return v54  ; v54 = 0
// }
// run: == true
