// test compile

vec3 main() {
    vec3 v = vec3(-1.0, 0.5, 2.0);
    return clamp(v, 0.0, 1.0);  // (0.0, 0.5, 1.0)
}

// function u0:0() -> f32, f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = fneg v0  ; v0 = 0x1.000000p0
//     v2 = f32const 0x1.000000p-1
//     v3 = f32const 0x1.000000p1
//     v4 = f32const 0.0
//     v5 = f32const 0x1.000000p0
//     v6 = fmax v1, v4  ; v4 = 0.0
//     v7 = fmax v2, v4  ; v2 = 0x1.000000p-1, v4 = 0.0
//     v8 = fmax v3, v4  ; v3 = 0x1.000000p1, v4 = 0.0
//     v9 = fmin v6, v5  ; v5 = 0x1.000000p0
//     v10 = fmin v7, v5  ; v5 = 0x1.000000p0
//     v11 = fmin v8, v5  ; v5 = 0x1.000000p0
//     return v9, v10, v11
//
// block1:
//     v12 = f32const 0.0
//     v13 = f32const 0.0
//     v14 = f32const 0.0
//     return v12, v13, v14  ; v12 = 0.0, v13 = 0.0, v14 = 0.0
// }
