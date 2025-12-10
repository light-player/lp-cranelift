// test compile
// test run
// target riscv32.fixed32

float main() {
    vec3 v = vec3(1.5, 2.5, 3.5);
    return v.x;  // 1.5
}

// function u0:0() -> i32 system_v {
// block0:
//     v4 = iconst.i32 0x0001_8000
//     v5 = iconst.i32 0x0002_8000
//     v6 = iconst.i32 0x0003_8000
//     return v4  ; v4 = 0x0001_8000
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
// run: ~= 1.5 (tolerance: 0.01)
