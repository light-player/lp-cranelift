// test compile
// test run

vec3 main() {
    return sqrt(vec3(4.0, 9.0, 16.0));  // Should return (2.0, 3.0, 4.0)
}

// function u0:0() -> f32, f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p2
//     v1 = f32const 0x1.200000p3
//     v2 = f32const 0x1.000000p4
//     v3 = sqrt v0  ; v0 = 0x1.000000p2
//     v4 = sqrt v1  ; v1 = 0x1.200000p3
//     v5 = sqrt v2  ; v2 = 0x1.000000p4
//     return v3, v4, v5
//
// block1:
//     v6 = f32const 0.0
//     v7 = f32const 0.0
//     v8 = f32const 0.0
//     return v6, v7, v8  ; v6 = 0.0, v7 = 0.0, v8 = 0.0
// }
// run: ≈ vec3(0, 0.00000000000000000000000000000000000000000088, 0) (tolerance: 0.01)
