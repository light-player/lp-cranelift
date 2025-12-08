// test compile

mat4 identity() {
    return mat4(1.0);
}

mat4 main() {
    return identity();
}

// function u0:0() -> f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v {
//     sig0 = () -> f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15 = call fn0()
//     return v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15
//
// block1:
//     v16 = f32const 0.0
//     v17 = f32const 0.0
//     v18 = f32const 0.0
//     v19 = f32const 0.0
//     v20 = f32const 0.0
//     v21 = f32const 0.0
//     v22 = f32const 0.0
//     v23 = f32const 0.0
//     v24 = f32const 0.0
//     v25 = f32const 0.0
//     v26 = f32const 0.0
//     v27 = f32const 0.0
//     v28 = f32const 0.0
//     v29 = f32const 0.0
//     v30 = f32const 0.0
//     v31 = f32const 0.0
//     return v16, v17, v18, v19, v20, v21, v22, v23, v24, v25, v26, v27, v28, v29, v30, v31  ; v16 = 0.0, v17 = 0.0, v18 = 0.0, v19 = 0.0, v20 = 0.0, v21 = 0.0, v22 = 0.0, v23 = 0.0, v24 = 0.0, v25 = 0.0, v26 = 0.0, v27 = 0.0, v28 = 0.0, v29 = 0.0, v30 = 0.0, v31 = 0.0
// }
