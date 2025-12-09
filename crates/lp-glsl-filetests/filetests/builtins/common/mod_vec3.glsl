// test compile
// test run
// target riscv32

bool main() {
    vec3 result = mod(vec3(7.0, 8.0, 9.0), vec3(3.0, 3.0, 4.0));  // (1.0, 2.0, 1.0)
    // Validate: sum = 1 + 2 + 1 = 4.0
    float sum = result.x + result.y + result.z;
    return sum > 3.99 && sum < 4.01;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = f32const 0x1.c00000p2
//     v1 = f32const 0x1.000000p3
//     v2 = f32const 0x1.200000p3
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.800000p1
//     v5 = f32const 0x1.000000p2
//     v6 = fdiv v0, v3  ; v0 = 0x1.c00000p2, v3 = 0x1.800000p1
//     v7 = floor v6
//     v8 = fmul v3, v7  ; v3 = 0x1.800000p1
//     v9 = fsub v0, v8  ; v0 = 0x1.c00000p2
//     v10 = fdiv v1, v4  ; v1 = 0x1.000000p3, v4 = 0x1.800000p1
//     v11 = floor v10
//     v12 = fmul v4, v11  ; v4 = 0x1.800000p1
//     v13 = fsub v1, v12  ; v1 = 0x1.000000p3
//     v14 = fdiv v2, v5  ; v2 = 0x1.200000p3, v5 = 0x1.000000p2
//     v15 = floor v14
//     v16 = fmul v5, v15  ; v5 = 0x1.000000p2
//     v17 = fsub v2, v16  ; v2 = 0x1.200000p3
//     v18 = fadd v9, v13
//     v19 = fadd v18, v17
//     v20 = f32const 0x1.feb852p1
//     v21 = fcmp gt v19, v20  ; v20 = 0x1.feb852p1
//     v22 = iconst.i8 1
//     v23 = iconst.i8 0
//     v24 = select v21, v22, v23  ; v22 = 1, v23 = 0
//     v25 = f32const 0x1.00a3d8p2
//     v26 = fcmp lt v19, v25  ; v25 = 0x1.00a3d8p2
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
