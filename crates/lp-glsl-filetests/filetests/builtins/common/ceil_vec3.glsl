// test compile
// test run

vec3 main() {
    return ceil(vec3(3.2, -2.7, 0.1));  // Should return (4.0, -2.0, 1.0)
}

// function u0:0() -> f32, f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.99999ap1
//     v1 = f32const 0x1.59999ap1
//     v2 = fneg v1  ; v1 = 0x1.59999ap1
//     v3 = f32const 0x1.99999ap-4
//     v4 = ceil v0  ; v0 = 0x1.99999ap1
//     v5 = ceil v2
//     v6 = ceil v3  ; v3 = 0x1.99999ap-4
//     return v4, v5, v6
//
// block1:
//     v7 = f32const 0.0
//     v8 = f32const 0.0
//     v9 = f32const 0.0
//     return v7, v8, v9  ; v7 = 0.0, v8 = 0.0, v9 = 0.0
// }
// run: ≈ vec3(0, 0.000000000000000000000000000000000000000000914, 0) (tolerance: 0.01)
