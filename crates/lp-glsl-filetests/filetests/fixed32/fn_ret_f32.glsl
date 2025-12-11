// test riscv32.fixed32
// test run

float get_value() {
    return 42.0;
}

float main() {
    return get_value();
}

// Generated CLIF
// function u0:0() -> f32 system_v {
//     sig0 = () -> f32 system_v
//     fn0 = colocated u0:0 sig0
// block0:
//     v0 = f32const 42.0
//     return v0
// block1:
//     v1 = f32const 0.0
//     return v1
// }
// function u0:1() -> f32 system_v {
// block0:
//     v0 = call fn0()
//     return v0
// block1:
//     v1 = f32const 0.0
//     return v1
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
//     sig0 = () -> i32 system_v
//     fn0 = colocated u0:0 sig0
// block0:
//     v0 = iconst.i32 0x002A_0000
//     return v0
// block1:
//     v1 = iconst.i32 0
//     return v1
// }
// function u0:1() -> i32 system_v {
// block0:
//     v0 = call fn0()
//     return v0
// block1:
//     v1 = iconst.i32 0
//     return v1
// }
// run: ≈ 42.0
