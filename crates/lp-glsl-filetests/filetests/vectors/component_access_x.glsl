// test compile
// test run

float main() {
    vec3 v = vec3(1.5, 2.5, 3.5);
    return v.x;  // 1.5
}

// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0x1.800000p0
//     v1 = f32const 0x1.400000p1
//     v2 = f32const 0x1.c00000p1
//     return v0  ; v0 = 0x1.800000p0
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 0 (tolerance: 0.01)
