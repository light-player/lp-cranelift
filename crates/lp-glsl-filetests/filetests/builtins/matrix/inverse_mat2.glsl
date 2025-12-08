// test compile
// test run

mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return inverse(m);
}

// function u0:0() -> f32, f32, f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = fmul v0, v3  ; v0 = 0x1.000000p0, v3 = 0x1.000000p2
//     v5 = fmul v2, v1  ; v2 = 0x1.800000p1, v1 = 0x1.000000p1
//     v6 = fsub v4, v5
//     v7 = f32const 0x1.000000p0
//     v8 = fdiv v7, v6  ; v7 = 0x1.000000p0
//     v9 = f32const 0.0
//     v10 = fsub v9, v1  ; v9 = 0.0, v1 = 0x1.000000p1
//     v11 = fsub v9, v2  ; v9 = 0.0, v2 = 0x1.800000p1
//     v12 = fmul v3, v8  ; v3 = 0x1.000000p2
//     v13 = fmul v10, v8
//     v14 = fmul v11, v8
//     v15 = fmul v0, v8  ; v0 = 0x1.000000p0
//     return v12, v13, v14, v15
//
// block1:
//     v16 = f32const 0.0
//     v17 = f32const 0.0
//     v18 = f32const 0.0
//     v19 = f32const 0.0
//     return v16, v17, v18, v19  ; v16 = 0.0, v17 = 0.0, v18 = 0.0, v19 = 0.0
// }
// run: ≈ mat2(0, 0, 0, 0.000000000000000000000000000000000000000000022) (tolerance: 0.01)
