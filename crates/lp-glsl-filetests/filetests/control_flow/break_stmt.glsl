// test compile
// test run

int main() {
    int sum = 0;
    int i = 0;
    while (i < 100) {
        if (i == 5) {
            break;
        }
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
//     v17 -> v8
//     v3 = iconst.i32 100
//     v4 = icmp slt v2, v3  ; v3 = 100
//     brif v4, block2, block3
//
// block2:
//     v5 = iconst.i32 5
//     v6 = icmp.i32 eq v2, v5  ; v5 = 5
//     brif v6, block4, block5(v8, v2)
//
// block4:
//     jump block3
//
// block6:
//     v13 = iconst.i32 0
//     v12 -> v13
//     v10 = iconst.i32 0
//     v9 -> v10
//     jump block5(v10, v13)  ; v10 = 0, v13 = 0
//
// block5(v7: i32, v11: i32):
//     v14 = iadd v7, v11
//     v15 = iconst.i32 1
//     v16 = iadd v11, v15  ; v15 = 1
//     jump block1(v16, v14)
//
// block3:
//     return v8
//
// block7:
//     v18 = iconst.i32 0
//     return v18  ; v18 = 0
// }
// run: == 10
