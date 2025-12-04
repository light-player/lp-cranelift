// test compile
// test run

int main() {
    int sum = 0;
    for (int i = 0; i < 5; i = i + 1) {
        sum = sum + i;
    }
    return sum;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0
//
// block1(v2: i32, v5: i32):
//     v3 = iconst.i32 5
//     v4 = icmp slt v2, v3  ; v3 = 5
//     brif v4, block2, block4
//
// block2:
//     v6 = iadd.i32 v5, v2
//     jump block3
//
// block3:
//     v7 = iconst.i32 1
//     v8 = iadd.i32 v2, v7  ; v7 = 1
//     jump block1(v8, v6)
//
// block4:
//     return v5
//
// block5:
//     v9 = iconst.i32 0
//     return v9  ; v9 = 0
// }
// run: == 10
