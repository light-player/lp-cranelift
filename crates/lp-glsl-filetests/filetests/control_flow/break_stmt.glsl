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
// block1(v2: i32, v14: i32):
//     v23 -> v14
//     v3 = iconst.i32 100
//     v4 = icmp slt v2, v3  ; v3 = 100
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v4, v5, v6  ; v5 = 1, v6 = 0
//     brif v7, block2, block3
//
// block2:
//     v8 = iconst.i32 5
//     v9 = icmp.i32 eq v2, v8  ; v8 = 5
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v9, v10, v11  ; v10 = 1, v11 = 0
//     brif v12, block4, block5(v14, v2)
//
// block4:
//     jump block3
//
// block6:
//     v19 = iconst.i32 0
//     v18 -> v19
//     v16 = iconst.i32 0
//     v15 -> v16
//     jump block5(v16, v19)  ; v16 = 0, v19 = 0
//
// block5(v13: i32, v17: i32):
//     v20 = iadd v13, v17
//     v21 = iconst.i32 1
//     v22 = iadd v17, v21  ; v21 = 1
//     jump block1(v22, v20)
//
// block3:
//     return v14
//
// block7:
//     v24 = iconst.i32 0
//     return v24  ; v24 = 0
// }
// run: == 10
