// test compile

mat3 identity() {
    return mat3(1.0);
}

mat3 main() {
    return identity();
}

// function u0:0() -> f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v {
//     sig0 = () -> f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0, v1, v2, v3, v4, v5, v6, v7, v8 = call fn0()
//     return v0, v1, v2, v3, v4, v5, v6, v7, v8
//
// block1:
//     v9 = f32const 0.0
//     v10 = f32const 0.0
//     v11 = f32const 0.0
//     v12 = f32const 0.0
//     v13 = f32const 0.0
//     v14 = f32const 0.0
//     v15 = f32const 0.0
//     v16 = f32const 0.0
//     v17 = f32const 0.0
//     return v9, v10, v11, v12, v13, v14, v15, v16, v17  ; v9 = 0.0, v10 = 0.0, v11 = 0.0, v12 = 0.0, v13 = 0.0, v14 = 0.0, v15 = 0.0, v16 = 0.0, v17 = 0.0
// }
