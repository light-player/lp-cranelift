// test compile

// target riscv32.fixed32
float add_floats(float a, float b) {
    return a + b;
}

float main() {
    int x = 5;
    return add_floats(x, 3.0);  // int→float conversion: 5.0 + 3.0 = 8.0
}

// function u0:0() -> f32 apple_aarch64 {
//     sig0 = (f32, f32) -> f32 apple_aarch64
//     fn0 = colocated u0:0 sig0
//
// block0:
//     v0 = iconst.i32 5
//     v1 = f32const 0x1.800000p1
//     v2 = fcvt_from_sint.f32 v0  ; v0 = 5
//     v3 = call fn0(v2, v1)  ; v1 = 0x1.800000p1
//     return v3
//
// block1:
//     v4 = f32const 0.0
//     return v4  ; v4 = 0.0
// }
