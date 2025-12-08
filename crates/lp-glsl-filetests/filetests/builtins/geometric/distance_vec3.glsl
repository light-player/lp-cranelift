// test compile

float main() {
    vec3 p0 = vec3(0.0, 0.0, 0.0);
    vec3 p1 = vec3(1.0, 2.0, 2.0);
    return distance(p0, p1);  // sqrt(1 + 4 + 4) = 3.0
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = f32const 0.0
//     v2 = f32const 0.0
//     v3 = f32const 0x1.000000p0
//     v4 = f32const 0x1.000000p1
//     v5 = f32const 0x1.000000p1
//     v6 = fsub v0, v3  ; v0 = 0.0, v3 = 0x1.000000p0
//     v7 = fsub v1, v4  ; v1 = 0.0, v4 = 0x1.000000p1
//     v8 = fsub v2, v5  ; v2 = 0.0, v5 = 0x1.000000p1
//     v9 = fmul v6, v6
//     v10 = fmul v7, v7
//     v11 = fadd v9, v10
//     v12 = fmul v8, v8
//     v13 = fadd v11, v12
//     v14 = sqrt v13
//     return v14
//
// block1:
//     v15 = f32const 0.0
//     return v15  ; v15 = 0.0
// }
