// test compile
// test run

int main() {
    int x = 5;
    if (x > 0) {
        return 42;
    }
    return 0;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 0
//     v2 = icmp sgt v0, v1  ; v0 = 5, v1 = 0
//     brif v2, block1, block2
//
// block1:
//     v3 = iconst.i32 42
//     return v3  ; v3 = 42
//
// block3:
//     jump block2
//
// block2:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
//
// block4:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
// run: == 42
