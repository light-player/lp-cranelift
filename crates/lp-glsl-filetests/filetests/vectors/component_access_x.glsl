// test compile
// test run
// target riscv32.fixed32

float main() {
    vec3 v = vec3(1.5, 2.5, 3.5);
    return v.x;  // 1.5
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0001_8000
//     v1 = iconst.i32 0x0002_8000
//     v2 = iconst.i32 0x0003_8000
//     return v0  ; v0 = 0x0001_8000
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: ~= 1.5 (tolerance: 0.01)
