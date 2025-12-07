// test compile

float main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    v.y = 5.0;  // v = (1.0, 5.0, 3.0)
    return v.y;  // 5.0
}

// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.400000p2
//     return v3  ; v3 = 0x1.400000p2
//
// block1:
//     v4 = f32const 0.0
//     return v4  ; v4 = 0.0
// }


