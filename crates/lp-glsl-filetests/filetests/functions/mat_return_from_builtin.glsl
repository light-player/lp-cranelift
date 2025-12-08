// test compile

mat3 transpose_mat(mat3 m) {
    return transpose(m);
}

mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose_mat(m);
}

// function u0:0() -> f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v {
//     sig0 = (f32, f32, f32, f32, f32, f32, f32, f32, f32) -> f32, f32, f32, f32, f32, f32, f32, f32, f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = f32const 0x1.400000p2
//     v5 = f32const 0x1.800000p2
//     v6 = f32const 0x1.c00000p2
//     v7 = f32const 0x1.000000p3
//     v8 = f32const 0x1.200000p3
//     v9, v10, v11, v12, v13, v14, v15, v16, v17 = call fn0(v0)  ; v0 = 0x1.000000p0
//     return v9, v10, v11, v12, v13, v14, v15, v16, v17
//
// block1:
//     v18 = f32const 0.0
//     v19 = f32const 0.0
//     v20 = f32const 0.0
//     v21 = f32const 0.0
//     v22 = f32const 0.0
//     v23 = f32const 0.0
//     v24 = f32const 0.0
//     v25 = f32const 0.0
//     v26 = f32const 0.0
//     return v18, v19, v20, v21, v22, v23, v24, v25, v26  ; v18 = 0.0, v19 = 0.0, v20 = 0.0, v21 = 0.0, v22 = 0.0, v23 = 0.0, v24 = 0.0, v25 = 0.0, v26 = 0.0
// }
