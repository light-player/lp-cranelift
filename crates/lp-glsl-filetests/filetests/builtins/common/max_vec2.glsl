// test compile

vec2 main() {
    vec2 a = vec2(1.0, 5.0);
    vec2 b = vec2(3.0, 2.0);
    return max(a, b);  // (3.0, 5.0)
}

// function u0:0() -> f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.400000p2
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p1
//     v4 = fmax v0, v2  ; v0 = 0x1.000000p0, v2 = 0x1.800000p1
//     v5 = fmax v1, v3  ; v1 = 0x1.400000p2, v3 = 0x1.000000p1
//     return v4, v5
//
// block1:
//     v6 = f32const 0.0
//     v7 = f32const 0.0
//     return v6, v7  ; v6 = 0.0, v7 = 0.0
// }
