// test compile
// test run
// target riscv32

bool main() {
    return true;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i8 1
//     return v0  ; v0 = 1
//
// block1:
//     v1 = iconst.i8 0
//     return v1  ; v1 = 0
// }
// run: == true
