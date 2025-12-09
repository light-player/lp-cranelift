// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return ceil(3.2);  // Should return 4.0
}

// function u0:0() -> f32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.99999ap1
//     v1 = ceil v0  ; v0 = 0x1.99999ap1
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 4.0
