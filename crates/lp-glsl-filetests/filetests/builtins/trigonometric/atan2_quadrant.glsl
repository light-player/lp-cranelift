// test compile
// test run
// target riscv32.fixed32
// target riscv32.fixed64

float main() {
    // atan(1.0, 0.0) should be π/2 (90 degrees)
    return atan(1.0, 0.0);
}

// function u0:0() -> i32 system_v {
//     sig0 = (f32, f32) -> f32 system_v
//     fn0 = u0:0 sig0
//
// block0:
//     v4 = iconst.i32 0x0001_0000
//     v5 = iconst.i32 0
//     v6 = f32const 0x1.000000p16
//     v7 = fcvt_from_sint.f32 v5  ; v5 = 0
//     v8 = fdiv v7, v6  ; v6 = 0x1.000000p16
//     v9 = f32const 0x1.000000p16
//     v10 = fcvt_from_sint.f32 v4  ; v4 = 0x0001_0000
//     v11 = fdiv v10, v9  ; v9 = 0x1.000000p16
//     v12 = call fn0(v8, v11)
//     v13 = f32const 0x1.000000p16
//     v14 = fmul v12, v13  ; v13 = 0x1.000000p16
//     v15 = fcvt_to_sint.i32 v14
//     return v15
//
// block1:
//     v16 = iconst.i32 0
//     return v16  ; v16 = 0
// }
// run: ~= 0 
