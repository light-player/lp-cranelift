// test compile

vec3 main() {
    vec3 v = vec3(5.0, 2.0, 7.0);
    return min(v, 4.0);  // (4.0, 2.0, 4.0)
}

// function u0:0() -> f32, f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.c00000p2
//     v3 = f32const 0x1.000000p2
//     v4 = fmin v0, v3  ; v0 = 0x1.400000p2, v3 = 0x1.000000p2
//     v5 = fmin v1, v3  ; v1 = 0x1.000000p1, v3 = 0x1.000000p2
//     v6 = fmin v2, v3  ; v2 = 0x1.c00000p2, v3 = 0x1.000000p2
//     return v4, v5, v6
//
// block1:
//     v7 = f32const 0.0
//     v8 = f32const 0.0
//     v9 = f32const 0.0
//     return v7, v8, v9  ; v7 = 0.0, v8 = 0.0, v9 = 0.0
// }
