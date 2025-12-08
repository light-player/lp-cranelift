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

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0
//
// block1(v2: i32, v14: i32):
//     v3 = iconst.i32 5
//     v4 = icmp slt v2, v3  ; v3 = 5
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v4, v5, v6  ; v5 = 1, v6 = 0
//     brif v7, block2, block4
//
// block2:
//     v8 = iconst.i32 2
//     v9 = icmp.i32 eq v2, v8  ; v8 = 2
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v9, v10, v11  ; v10 = 1, v11 = 0
//     brif v12, block5, block6(v14, v2)
//
// block5:
//     jump block3(v2, v14)
//
// block7:
//     v19 = iconst.i32 0
//     v18 -> v19
//     v16 = iconst.i32 0
//     v15 -> v16
//     jump block6(v16, v19)  ; v16 = 0, v19 = 0
//
// block6(v13: i32, v17: i32):
//     v20 = iadd v13, v17
//     jump block3(v17, v20)
//
// block3(v21: i32, v24: i32):
//     v22 = iconst.i32 1
//     v23 = iadd v21, v22  ; v22 = 1
//     jump block1(v23, v24)
//
// block4:
//     return v14
//
// block8:
//     v25 = iconst.i32 0
//     return v25  ; v25 = 0
// }
// run: == 8
