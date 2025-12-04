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

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 10
//     v2 = icmp sgt v0, v1  ; v0 = 5, v1 = 10
//     brif v2, block1, block3
//
// block1:
//     v3 = iconst.i32 1
//     jump block2(v3)  ; v3 = 1
//
// block3:
//     v4 = iconst.i32 0
//     jump block2(v4)  ; v4 = 0
//
// block2(v5: i32):
//     return v5
//
// block4:
//     v6 = iconst.i32 0
//     return v6  ; v6 = 0
// }
// run: == 0
