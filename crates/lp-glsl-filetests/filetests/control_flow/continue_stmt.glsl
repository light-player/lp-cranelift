// test compile
// test run

int main() {
    int sum = 0;
    for (int i = 0; i < 5; i = i + 1) {
        if (i == 2) {
            continue;
        }
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
// block1(v2: i32, v8: i32):
//     v3 = iconst.i32 5
//     v4 = icmp slt v2, v3  ; v3 = 5
//     brif v4, block2, block4
//
// block2:
//     v5 = iconst.i32 2
//     v6 = icmp.i32 eq v2, v5  ; v5 = 2
//     brif v6, block5, block6(v8, v2)
//
// block5:
//     jump block3(v2, v8)
//
// block7:
//     v13 = iconst.i32 0
//     v12 -> v13
//     v10 = iconst.i32 0
//     v9 -> v10
//     jump block6(v10, v13)  ; v10 = 0, v13 = 0
//
// block6(v7: i32, v11: i32):
//     v14 = iadd v7, v11
//     jump block3(v11, v14)
//
// block3(v15: i32, v18: i32):
//     v16 = iconst.i32 1
//     v17 = iadd v15, v16  ; v16 = 1
//     jump block1(v17, v18)
//
// block4:
//     return v8
//
// block8:
//     v19 = iconst.i32 0
//     return v19  ; v19 = 0
// }
// run: == 8
