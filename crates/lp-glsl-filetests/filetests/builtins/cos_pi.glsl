// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    float pi = 3.141592654;
    return cos(pi);
}

// function u0:0() -> f32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v0 = f32const 0x1.921fb6p1
//     v1 = call fn0(v0)  ; v0 = 0x1.921fb6p1
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= -1
