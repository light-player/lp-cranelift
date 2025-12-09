// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return sqrt(16.0);  // 4.0
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.000000p4
//     v1 = sqrt v0  ; v0 = 0x1.000000p4
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 4
