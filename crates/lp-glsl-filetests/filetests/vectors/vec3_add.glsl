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
//     v11 = iconst.i32 0x0001_0000
//     v12 = iconst.i32 0x0002_0000
//     v13 = iconst.i32 0x0003_0000
//     v14 = iconst.i32 0x0004_0000
//     v15 = iconst.i32 0x0005_0000
//     v16 = iconst.i32 0x0006_0000
//     v17 = iadd v11, v14  ; v11 = 0x0001_0000, v14 = 0x0004_0000
//     v18 = iadd v12, v15  ; v12 = 0x0002_0000, v15 = 0x0005_0000
//     v19 = iadd v13, v16  ; v13 = 0x0003_0000, v16 = 0x0006_0000
//     v9 = iconst.i32 1
//     return v9  ; v9 = 1
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }
// run: == 1
