// test compile

mat2 identity() {
    return mat2(1.0);
}

mat2 main() {
    return identity();
}

// function u0:0() -> f32, f32, f32, f32 system_v {
//     sig0 = () -> f32, f32, f32, f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0, v1, v2, v3 = call fn0()
//     return v0, v1, v2, v3
//
// block1:
//     v4 = f32const 0.0
//     v5 = f32const 0.0
//     v6 = f32const 0.0
//     v7 = f32const 0.0
//     return v4, v5, v6, v7  ; v4 = 0.0, v5 = 0.0, v6 = 0.0, v7 = 0.0
// }
