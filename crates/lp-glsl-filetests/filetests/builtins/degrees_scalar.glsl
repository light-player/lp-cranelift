// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    float pi = 3.141592654;
    return degrees(pi);  // Should be 180.0
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.921fb6p1
//     v1 = f32const 0x1.ca5dc2p5
//     v2 = fmul v0, v1  ; v0 = 0x1.921fb6p1, v1 = 0x1.ca5dc2p5
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
// run: ~= 180
