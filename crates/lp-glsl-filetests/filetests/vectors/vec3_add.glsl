// test compile
// test run
// target riscv32.fixed32

int main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    vec3 c = a + b;  // (5.0, 7.0, 9.0)
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0001_0000
//     v1 = iconst.i32 0x0002_0000
//     v2 = iconst.i32 0x0003_0000
//     v3 = iconst.i32 0x0004_0000
//     v4 = iconst.i32 0x0005_0000
//     v5 = iconst.i32 0x0006_0000
//     v6 = iadd v0, v3  ; v0 = 0x0001_0000, v3 = 0x0004_0000
//     v7 = iadd v1, v4  ; v1 = 0x0002_0000, v4 = 0x0005_0000
//     v8 = iadd v2, v5  ; v2 = 0x0003_0000, v5 = 0x0006_0000
//     v9 = iconst.i32 1
//     return v9  ; v9 = 1
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }
// run: == 1
