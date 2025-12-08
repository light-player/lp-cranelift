// test compile
// test run

float main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return determinant(m);
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = fmul v0, v3  ; v0 = 0x1.000000p0, v3 = 0x1.000000p2
//     v5 = fmul v2, v1  ; v2 = 0x1.800000p1, v1 = 0x1.000000p1
//     v6 = fsub v4, v5
//     return v6
//
// block1:
//     v7 = f32const 0.0
//     return v7  ; v7 = 0.0
// }
// run: ~= -2.0 (tolerance: 0.01)  // 1*4 - 2*3 = 4 - 6 = -2
