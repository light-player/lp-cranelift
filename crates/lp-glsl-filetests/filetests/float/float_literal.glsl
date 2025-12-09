// test compile
// test run
// target riscv32

float main() {
    return 3.14;
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.91eb86p1
//     return v0  ; v0 = 0x1.91eb86p1
//
// block1:
//     v1 = f32const 0.0
//     return v1  ; v1 = 0.0
// }
// run: ~= 3.14 (tolerance: 0.01)
