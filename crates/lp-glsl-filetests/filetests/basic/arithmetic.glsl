// test compile
// test run
// target riscv32

int main() {
    int a = 10;
    int b = 20;
    return a + b;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 20
//     v2 = iadd v0, v1  ; v0 = 10, v1 = 20
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
// run: == 30
