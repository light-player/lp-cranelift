// test riscv32.fixed32
// test run

float main() {
    return min(3.5, 7.2);
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.c00000p1
//     v1 = f32const 0x1.ccccccp2
//     v2 = fmin v0, v1  ; v0 = 0x1.c00000p1, v1 = 0x1.ccccccp2
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0003_8000
//     v1 = iconst.i32 0x0007_3333
//     v2 = icmp slt v0, v1  ; v0 = 0x0003_8000, v1 = 0x0007_3333
//     v3 = select v2, v0, v1  ; v0 = 0x0003_8000, v1 = 0x0007_3333
//     return v3
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
// run: ≈ 3.5
