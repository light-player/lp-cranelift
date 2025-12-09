// test compile
// test run
// target riscv32

bool main() {
    // smoothstep(0, 10, 5): t = 0.5, result = t²(3-2t) = 0.25*2 = 0.5
    float result = smoothstep(0.0, 10.0, 5.0);
    return result > 0.49 && result < 0.51;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0x1.400000p3
//     v2 = f32const 0x1.400000p2
//     v3 = f32const 0.0
//     v4 = f32const 0x1.000000p0
//     v5 = f32const 0x1.000000p1
//     v6 = f32const 0x1.800000p1
//     v7 = fsub v2, v0  ; v2 = 0x1.400000p2, v0 = 0.0
//     v8 = fsub v1, v0  ; v1 = 0x1.400000p3, v0 = 0.0
//     v9 = fdiv v7, v8
//     v10 = fmax v9, v3  ; v3 = 0.0
//     v11 = fmin v10, v4  ; v4 = 0x1.000000p0
//     v12 = fmul v11, v11
//     v13 = fmul v5, v11  ; v5 = 0x1.000000p1
//     v14 = fsub v6, v13  ; v6 = 0x1.800000p1
//     v15 = fmul v12, v14
//     v16 = f32const 0x1.f5c290p-2
//     v17 = fcmp gt v15, v16  ; v16 = 0x1.f5c290p-2
//     v18 = iconst.i8 1
//     v19 = iconst.i8 0
//     v20 = select v17, v18, v19  ; v18 = 1, v19 = 0
//     v21 = f32const 0x1.051eb8p-1
//     v22 = fcmp lt v15, v21  ; v21 = 0x1.051eb8p-1
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
