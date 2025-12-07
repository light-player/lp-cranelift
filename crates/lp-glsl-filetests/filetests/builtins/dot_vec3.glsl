// test compile
// test run

float main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    return dot(a, b);  // 1*4 + 2*5 + 3*6 = 32.0
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = f32const 0x1.400000p2
//     v5 = f32const 0x1.800000p2
//     v6 = fmul v0, v3  ; v0 = 0x1.000000p0, v3 = 0x1.000000p2
//     v7 = fmul v1, v4  ; v1 = 0x1.000000p1, v4 = 0x1.400000p2
//     v8 = fadd v6, v7
//     v9 = fmul v2, v5  ; v2 = 0x1.800000p1, v5 = 0x1.800000p2
//     v10 = fadd v8, v9
//     return v10
//
// block1:
//     v11 = f32const 0.0
//     return v11  ; v11 = 0.0
// }
// run: ~= 0 (tolerance: 0.01)
