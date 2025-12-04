// test compile
// test run

int main() {
    return 42;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 42
//     return v0  ; v0 = 42
//
// block1:
//     v1 = iconst.i32 0
//     return v1  ; v1 = 0
// }
// run: == 42
