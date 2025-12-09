// test compile
// test run
// target riscv32

vec2 main() {
    vec2 angles = vec2(0.0, 3.141592654); // 0, π
    return cos(angles);
}

// function u0:0(i32 sret) system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0(v0: i32):
//     v1 = f32const 0.0
//     v2 = f32const 0x1.921fb6p1
//     v3 = call fn0(v1)  ; v1 = 0.0
//     v4 = call fn0(v2)  ; v2 = 0x1.921fb6p1
//     store notrap aligned v3, v0
//     store notrap aligned v4, v0+4
//     return
//
// block1:
//     v5 = f32const 0.0
//     store notrap aligned v5, v0  ; v5 = 0.0
//     v6 = f32const 0.0
//     store notrap aligned v6, v0+4  ; v6 = 0.0
//     return
// }
// run: ≈ vec2(1.0, -1.0) (tolerance: 0.001)
