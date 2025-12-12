// test compile
// test transform.fixed32
// test run
// target riscv32.fixed32

float add_float(float a, float b) {
    return a + b;
}

// #compile: initial.clif
// function u0:0(f32, f32) -> f32 fast {
// block0(v0: f32, v1: f32):
//     v2 = fadd v0, v1
//     return v2
// }
//
// #transform: fixed32.clif
// function u0:0(i32, i32) -> i32 fast {
// block0(v0: i32, v1: i32):
//     v2 = iadd v0, v1
//     return v2
// }
//
// #run: add_float(0.0, 0.0) ~= 0.0
// #run: add_float(1.5, 2.5) ~= 4.0

int add_int(int a, int b) {
    return a + b;
}

// #run: add_int(0, 0) == 0
// #run: add_int(1, 2) == 3
