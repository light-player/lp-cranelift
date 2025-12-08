// test compile
// test run

mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}

// function u0:0() -> f32, f32, f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = f32const 0x1.000000p1
//     v5 = f32const 0x1.000000p1
//     v6 = f32const 0x1.000000p1
//     v7 = f32const 0x1.000000p1
//     v8 = fmul v0, v4  ; v0 = 0x1.000000p0, v4 = 0x1.000000p1
//     v9 = fmul v1, v5  ; v1 = 0x1.000000p1, v5 = 0x1.000000p1
//     v10 = fmul v2, v6  ; v2 = 0x1.800000p1, v6 = 0x1.000000p1
//     v11 = fmul v3, v7  ; v3 = 0x1.000000p2, v7 = 0x1.000000p1
//     return v8, v9, v10, v11
//
// block1:
//     v12 = f32const 0.0
//     v13 = f32const 0.0
//     v14 = f32const 0.0
//     v15 = f32const 0.0
//     return v12, v13, v14, v15  ; v12 = 0.0, v13 = 0.0, v14 = 0.0, v15 = 0.0
// }
// run: ≈ mat2(0, 0, 0, 0.000000000000000000000000000000000000000000017) (tolerance: 0.01)
