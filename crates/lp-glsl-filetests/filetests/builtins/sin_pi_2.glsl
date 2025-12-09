// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    float pi_2 = 1.570796327;  // π/2
    return sin(pi_2);
}

// function u0:0() -> f32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.921fb6p0
//     v1 = call fn0(v0)  ; v0 = 0x1.921fb6p0
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 1
