// test compile
// test run

mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}

// function u0:0(i64 sret) apple_aarch64 {
// block0(v0: i64):
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = f32const 0x1.000000p2
//     v5 = f32const 0x1.000000p1
//     v6 = f32const 0x1.000000p1
//     v7 = f32const 0x1.000000p1
//     v8 = f32const 0x1.000000p1
//     v9 = fmul v1, v5  ; v1 = 0x1.000000p0, v5 = 0x1.000000p1
//     v10 = fmul v2, v6  ; v2 = 0x1.000000p1, v6 = 0x1.000000p1
//     v11 = fmul v3, v7  ; v3 = 0x1.800000p1, v7 = 0x1.000000p1
//     v12 = fmul v4, v8  ; v4 = 0x1.000000p2, v8 = 0x1.000000p1
//     store notrap aligned v9, v0
//     store notrap aligned v10, v0+4
//     store notrap aligned v11, v0+8
//     store notrap aligned v12, v0+12
//     return
//
// block1:
//     v13 = f32const 0.0
//     store notrap aligned v13, v0  ; v13 = 0.0
//     v14 = f32const 0.0
//     store notrap aligned v14, v0+4  ; v14 = 0.0
//     v15 = f32const 0.0
//     store notrap aligned v15, v0+8  ; v15 = 0.0
//     v16 = f32const 0.0
//     store notrap aligned v16, v0+12  ; v16 = 0.0
//     return
// }
// run: ≈ mat2(2, 4, 6, 8) (tolerance: 0.01)
