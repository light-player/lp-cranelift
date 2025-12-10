// test compile
// test run
// target riscv32.fixed32

int main() {
    vec3 v = vec3(1.0, 2.0, 3.0);
    return 1;  // Just test construction succeeds
}

// function u0:0() -> i32 system_v {
// block0:
//     v5 = iconst.i32 0x0001_0000
//     v6 = iconst.i32 0x0002_0000
//     v7 = iconst.i32 0x0003_0000
//     v3 = iconst.i32 1
//     return v3  ; v3 = 1
//
// block1:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
// }
// run: == 1
