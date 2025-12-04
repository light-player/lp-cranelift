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

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0
//
// block1(v2: i32, v14: i32):
//     v15 -> v2
//     v3 = iconst.i32 3
//     v4 = icmp slt v2, v3  ; v3 = 3
//     brif v4, block2, block4
//
// block2:
//     v5 = iconst.i32 0
//     jump block5(v5, v14)  ; v5 = 0
//
// block5(v6: i32, v9: i32):
//     v7 = iconst.i32 3
//     v8 = icmp slt v6, v7  ; v7 = 3
//     brif v8, block6, block8
//
// block6:
//     v10 = iconst.i32 1
//     v11 = iadd.i32 v9, v10  ; v10 = 1
//     jump block7
//
// block7:
//     v12 = iconst.i32 1
//     v13 = iadd.i32 v6, v12  ; v12 = 1
//     jump block5(v13, v11)
//
// block8:
//     jump block3
//
// block3:
//     v16 = iconst.i32 1
//     v17 = iadd.i32 v2, v16  ; v16 = 1
//     jump block1(v17, v9)
//
// block4:
//     return v14
//
// block9:
//     v18 = iconst.i32 0
//     return v18  ; v18 = 0
// }
// run: == 9
