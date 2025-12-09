// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    return cos(0.0);
}

// function u0:0() -> i32 system_v {
//     sig0 = (f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v3 = iconst.i32 0
//     v4 = f32const 0x1.000000p16
//     v5 = fcvt_from_sint.f32 v3  ; v3 = 0
//     v6 = fdiv v5, v4  ; v4 = 0x1.000000p16
//     v7 = call fn0(v6)
//     v8 = f32const 0x1.000000p16
//     v9 = fmul v7, v8  ; v8 = 0x1.000000p16
//     v10 = fcvt_to_sint.i32 v9
//     return v10
//
// block1:
//     v11 = iconst.i32 0
//     return v11  ; v11 = 0
// }
// run: ~= 1 
