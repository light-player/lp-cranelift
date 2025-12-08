// test compile
// test run

int main() {
    int x = 5;
    int result;
    if (x > 10) {
        result = 1;
    } else {
        result = 0;
    }
    return result;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 10
//     v2 = icmp sgt v0, v1  ; v0 = 5, v1 = 10
//     v3 = iconst.i8 1
//     v4 = iconst.i8 0
//     v5 = select v2, v3, v4  ; v3 = 1, v4 = 0
//     brif v5, block1, block3
//
// block1:
//     v6 = iconst.i32 1
//     jump block2(v6)  ; v6 = 1
//
// block3:
//     v7 = iconst.i32 0
//     jump block2(v7)  ; v7 = 0
//
// block2(v8: i32):
//     return v8
//
// block4:
//     v9 = iconst.i32 0
//     return v9  ; v9 = 0
// }
// run: == 0
