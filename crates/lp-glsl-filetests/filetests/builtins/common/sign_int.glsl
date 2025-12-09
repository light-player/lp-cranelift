// test compile
// test run

int main() {
    int a = sign(-5);  // -1
    int b = sign(0);   // 0
    int c = sign(5);   // 1
    // Validate sum: -1 + 0 + 1 = 0
    return a + b + c;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 5
//     v1 = ineg v0  ; v0 = 5
//     v2 = iconst.i32 0
//     v3 = iconst.i32 1
//     v4 = iconst.i32 -1
//     v5 = icmp sgt v1, v2  ; v2 = 0
//     v6 = icmp slt v1, v2  ; v2 = 0
//     v7 = select v5, v3, v2  ; v3 = 1, v2 = 0
//     v8 = select v6, v4, v7  ; v4 = -1
//     v9 = iconst.i32 0
//     v10 = iconst.i32 0
//     v11 = iconst.i32 1
//     v12 = iconst.i32 -1
//     v13 = icmp sgt v9, v10  ; v9 = 0, v10 = 0
//     v14 = icmp slt v9, v10  ; v9 = 0, v10 = 0
//     v15 = select v13, v11, v10  ; v11 = 1, v10 = 0
//     v16 = select v14, v12, v15  ; v12 = -1
//     v17 = iconst.i32 5
//     v18 = iconst.i32 0
//     v19 = iconst.i32 1
//     v20 = iconst.i32 -1
//     v21 = icmp sgt v17, v18  ; v17 = 5, v18 = 0
//     v22 = icmp slt v17, v18  ; v17 = 5, v18 = 0
//     v23 = select v21, v19, v18  ; v19 = 1, v18 = 0
//     v24 = select v22, v20, v23  ; v20 = -1
//     v25 = iadd v8, v16
//     v26 = iadd v25, v24
//     return v26
//
// block1:
//     v27 = iconst.i32 0
//     return v27  ; v27 = 0
// }
// run: == 0
