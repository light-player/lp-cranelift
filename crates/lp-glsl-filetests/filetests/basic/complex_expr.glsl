// test compile
// test run

int main() {
    int a = 5;
    int b = 3;
    int c = 2;
    return (a + b) * c - 4;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 3
//     v2 = iconst.i32 2
//     v3 = iadd v0, v1  ; v0 = 5, v1 = 3
//     v4 = imul v3, v2  ; v2 = 2
//     v5 = iconst.i32 4
//     v6 = isub v4, v5  ; v5 = 4
//     return v6
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
// run: == 12
