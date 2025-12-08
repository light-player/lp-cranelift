// test compile
// test run

int main() {
    int sum = 0;
    for (int i = 0; i < 3; i = i + 1) {
        for (int j = 0; j < 3; j = j + 1) {
            sum = sum + 1;
        }
    }
    return sum;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0
//
// block1(v2: i32, v20: i32):
//     v21 -> v2
//     v3 = iconst.i32 3
//     v4 = icmp slt v2, v3  ; v3 = 3
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v4, v5, v6  ; v5 = 1, v6 = 0
//     brif v7, block2, block4
//
// block2:
//     v8 = iconst.i32 0
//     jump block5(v8, v20)  ; v8 = 0
//
// block5(v9: i32, v15: i32):
//     v10 = iconst.i32 3
//     v11 = icmp slt v9, v10  ; v10 = 3
//     v12 = iconst.i8 1
//     v13 = iconst.i8 0
//     v14 = select v11, v12, v13  ; v12 = 1, v13 = 0
//     brif v14, block6, block8
//
// block6:
//     v16 = iconst.i32 1
//     v17 = iadd.i32 v15, v16  ; v16 = 1
//     jump block7
//
// block7:
//     v18 = iconst.i32 1
//     v19 = iadd.i32 v9, v18  ; v18 = 1
//     jump block5(v19, v17)
//
// block8:
//     jump block3
//
// block3:
//     v22 = iconst.i32 1
//     v23 = iadd.i32 v2, v22  ; v22 = 1
//     jump block1(v23, v15)
//
// block4:
//     return v20
//
// block9:
//     v24 = iconst.i32 0
//     return v24  ; v24 = 0
// }
// run: == 9
