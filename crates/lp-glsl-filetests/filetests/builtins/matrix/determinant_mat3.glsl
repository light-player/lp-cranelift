// test compile
// test run

float main() {
    mat3 m = mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
    return determinant(m);
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0.0
//     v2 = f32const 0.0
//     v3 = f32const 0.0
//     v4 = f32const 0x1.000000p0
//     v5 = f32const 0.0
//     v6 = f32const 0.0
//     v7 = f32const 0.0
//     v8 = f32const 0x1.000000p0
//     v9 = fmul v4, v8  ; v4 = 0x1.000000p0, v8 = 0x1.000000p0
//     v10 = fmul v7, v5  ; v7 = 0.0, v5 = 0.0
//     v11 = fsub v9, v10
//     v12 = fmul v0, v11  ; v0 = 0x1.000000p0
//     v13 = fmul v1, v8  ; v1 = 0.0, v8 = 0x1.000000p0
//     v14 = fmul v7, v2  ; v7 = 0.0, v2 = 0.0
//     v15 = fsub v13, v14
//     v16 = fmul v3, v15  ; v3 = 0.0
//     v17 = fmul v1, v5  ; v1 = 0.0, v5 = 0.0
//     v18 = fmul v4, v2  ; v4 = 0x1.000000p0, v2 = 0.0
//     v19 = fsub v17, v18
//     v20 = fmul v6, v19  ; v6 = 0.0
//     v21 = fsub v12, v16
//     v22 = fadd v21, v20
//     return v22
//
// block1:
//     v23 = f32const 0.0
//     return v23  ; v23 = 0.0
// }
// run: ~= 1.0 (tolerance: 0.01)  // Identity matrix has determinant 1
