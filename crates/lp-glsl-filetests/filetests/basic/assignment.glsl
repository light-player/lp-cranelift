// test compile
// test run

int main() {
    int x = 5;
    int y = x + 10;
    int z = y * 2;
    return z;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 10
//     v2 = iadd v0, v1  ; v0 = 5, v1 = 10
//     v3 = iconst.i32 2
//     v4 = imul v2, v3  ; v3 = 2
//     return v4
//
// block1:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
// run: == 30
