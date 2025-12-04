// test compile
// test run

int main() {
    int x = 10;
    return -x;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 10
//     v1 = ineg v0  ; v0 = 10
//     return v1
//
// block1:
//     v2 = iconst.i32 0
//     return v2  ; v2 = 0
// }
// run: == -10
