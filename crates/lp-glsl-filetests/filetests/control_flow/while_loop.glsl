// test compile
// test run

int main() {
    int sum = 0;
    int i = 0;
    while (i < 5) {
        sum = sum + i;
        i = i + 1;
    }
    return sum;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0
//
// block1(v2: i32, v8: i32):
//     v3 = iconst.i32 5
//     v4 = icmp slt v2, v3  ; v3 = 5
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v4, v5, v6  ; v5 = 1, v6 = 0
//     brif v7, block2, block3
//
// block2:
//     v9 = iadd.i32 v8, v2
//     v10 = iconst.i32 1
//     v11 = iadd.i32 v2, v10  ; v10 = 1
//     jump block1(v11, v9)
//
// block3:
//     return v8
//
// block4:
//     v12 = iconst.i32 0
//     return v12  ; v12 = 0
// }
// run: == 10
