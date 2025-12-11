// test riscv32.fixed32
// test run

float add_one(float x) {
    return x + 1.0;
}

float main() {
    return add_one(5.0);
}

// Generated CLIF
// function u0:0(f32) -> f32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = colocated u0:0 sig0
// block0(v0: f32):
//     v1 = f32const 1.0
//     v2 = fadd v0, v1
//     return v2
// block1:
//     v3 = f32const 0.0
//     return v3
// }
// function u0:1() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = call fn0(v0)
//     return v1
// block1:
//     v2 = f32const 0.0
//     return v2
// }
//
// Transformed CLIF
// function u0:0(i32) -> i32 system_v {
//     sig0 = (i32) -> i32 system_v
//     fn0 = colocated u0:0 sig0
// block0(v0: i32):
//     v1 = iconst.i32 0x0001_0000
//     v2 = iadd v0, v1
//     return v2
// block1:
//     v3 = iconst.i32 0
//     return v3
// }
// function u0:1() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0005_0000
//     v1 = call fn0(v0)
//     return v1
// block1:
//     v2 = iconst.i32 0
//     return v2
// }
// run: ≈ 6.0
