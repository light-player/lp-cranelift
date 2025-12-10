// test compile

// target riscv32.fixed32
float double_it(float x) {
    return x * 2.0;
}

float triple_it(float x) {
    return x * 3.0;
}

float main() {
    float a = double_it(5.0);   // 10.0
    float b = triple_it(a);     // 30.0
    return b;
}

// function u0:0() -> f32 apple_aarch64 {
//     sig0 = (f32) -> f32 apple_aarch64
//     sig1 = (f32) -> f32 apple_aarch64
//     fn0 = colocated u0:0 sig0
//     fn1 = colocated u0:1 sig1
//
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = call fn0(v0)  ; v0 = 0x1.400000p2
//     v2 = call fn1(v1)
//     return v2
//
// block1:
//     v3 = f32const 0.0
//     return v3  ; v3 = 0.0
// }
