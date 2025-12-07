// test compile

vec3 scale(vec3 v, float s) {
    return v * s;
}

vec3 main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return scale(v, 2.0);  // (2.0, 4.0, 6.0)
}

// function u0:0() -> f32, f32, f32 system_v {
//     sig0 = (f32, f32, f32, f32) -> f32, f32, f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p1
//     v4, v5, v6 = call fn0(v0, v1, v2, v3)  ; v0 = 0x1.000000p0, v1 = 0x1.000000p1, v2 = 0x1.800000p1, v3 = 0x1.000000p1
//     return v4, v5, v6
//
// block1:
//     v7 = f32const 0.0
//     v8 = f32const 0.0
//     v9 = f32const 0.0
//     return v7, v8, v9  ; v7 = 0.0, v8 = 0.0, v9 = 0.0
// }
