// test riscv32.fixed32
// test run

float main() {
    return 10.0 - 3.5;
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.400000p3
//     v1 = f32const 0x1.c00000p1
//     v2 = fsub v0, v1  ; v0 = 0x1.400000p3, v1 = 0x1.c00000p1
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
//     v0 = iconst.i32 0x000a_0000
//     v1 = iconst.i32 0x0003_8000
//     v2 = isub v0, v1  ; v0 = 0x000a_0000, v1 = 0x0003_8000
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ≈ 6.5
