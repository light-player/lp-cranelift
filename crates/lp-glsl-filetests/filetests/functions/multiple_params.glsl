// test compile

float add3(float a, float b, float c) {
    return a + b + c;
}

float main() {
    return add3(1.0, 2.0, 3.0);  // 6.0
}

// function u0:0() -> f32 system_v {
//     sig0 = (f32, f32, f32) -> f32 system_v
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = call fn0(v0, v1, v2)  ; v0 = 0x1.000000p0, v1 = 0x1.000000p1, v2 = 0x1.800000p1
//     return v3
//
// block1:
//     v4 = f32const 0.0
//     return v4  ; v4 = 0.0
// }
