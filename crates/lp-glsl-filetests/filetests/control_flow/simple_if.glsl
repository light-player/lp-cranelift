// test compile
// test run

int main() {
    int x = 5;
    if (x > 0) {
        x = 10;
    }
    return x;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 0
//     v2 = icmp sgt v0, v1  ; v0 = 5, v1 = 0
//     brif v2, block1, block2(v0)  ; v0 = 5
//
// block1:
//     v3 = iconst.i32 10
//     jump block2(v3)  ; v3 = 10
//
// block2(v4: i32):
//     return v4
//
// block3:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
// run: == 10
