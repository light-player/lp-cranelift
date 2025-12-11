// test compile
// test run
// target riscv32.fixed32

vec2 main() {
    vec2 deg = vec2(90.0, 45.0);
    return radians(deg);
}

// function u0:0() -> f32, f32 system_v {
// block0:
//     v0 = f32const 0x1.680000p6
//     v1 = f32const 0x1.680000p5
//     v2 = f32const 0x1.1df46ap-6
//     v3 = fmul v0, v2  ; v0 = 0x1.680000p6, v2 = 0x1.1df46ap-6
//     v4 = fmul v1, v2  ; v1 = 0x1.680000p5, v2 = 0x1.1df46ap-6
//     return v3, v4
//
// block1:
//     v5 = f32const 0.0
//     v6 = f32const 0.0
//     return v5, v6  ; v5 = 0.0, v6 = 0.0
// }
// run: ≈ vec2(1.5708, 0.7854) (tolerance: 0.01)
