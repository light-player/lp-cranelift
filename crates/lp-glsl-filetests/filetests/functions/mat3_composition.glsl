// test compile

mat3 scale(mat3 m, float s) {
    return m * s;
}

mat3 main() {
    mat3 m = mat3(1.0);
    return scale(m, 2.0);
}

// function u0:0() -> f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v {
//     sig0 = (f32, f32, f32, f32, f32, f32, f32, f32, f32, f32) -> f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0.0
//     v2 = f32const 0x1.000000p1
//     v3, v4, v5, v6, v7, v8, v9, v10, v11 = call fn0(v0, v1)  ; v0 = 0x1.000000p0, v1 = 0.0
//     return v3, v4, v5, v6, v7, v8, v9, v10, v11
//
// block1:
//     v12 = f32const 0.0
//     v13 = f32const 0.0
//     v14 = f32const 0.0
//     v15 = f32const 0.0
//     v16 = f32const 0.0
//     v17 = f32const 0.0
//     v18 = f32const 0.0
//     v19 = f32const 0.0
//     v20 = f32const 0.0
//     return v12, v13, v14, v15, v16, v17, v18, v19, v20  ; v12 = 0.0, v13 = 0.0, v14 = 0.0, v15 = 0.0, v16 = 0.0, v17 = 0.0, v18 = 0.0, v19 = 0.0, v20 = 0.0
// }
