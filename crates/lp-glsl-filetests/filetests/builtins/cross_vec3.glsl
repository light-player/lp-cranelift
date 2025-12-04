// test compile

vec3 main() {
    vec3 x = vec3(1.0, 0.0, 0.0);
    vec3 y = vec3(0.0, 1.0, 0.0);
    return cross(x, y);  // = (0.0, 0.0, 1.0)
}

// function u0:0() -> f32, f32, f32 fast {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0.0
//     v2 = f32const 0.0
//     v3 = f32const 0.0
//     v4 = f32const 0x1.000000p0
//     v5 = f32const 0.0
//     v6 = fmul v1, v5  ; v1 = 0.0, v5 = 0.0
//     v7 = fmul v2, v4  ; v2 = 0.0, v4 = 0x1.000000p0
//     v8 = fsub v6, v7
//     v9 = fmul v2, v3  ; v2 = 0.0, v3 = 0.0
//     v10 = fmul v0, v5  ; v0 = 0x1.000000p0, v5 = 0.0
//     v11 = fsub v9, v10
//     v12 = fmul v0, v4  ; v0 = 0x1.000000p0, v4 = 0x1.000000p0
//     v13 = fmul v1, v3  ; v1 = 0.0, v3 = 0.0
//     v14 = fsub v12, v13
//     return v8
//
// block1:
//     v15 = f32const 0.0
//     v16 = f32const 0.0
//     v17 = f32const 0.0
//     return v15, v16, v17  ; v15 = 0.0, v16 = 0.0, v17 = 0.0
// }
