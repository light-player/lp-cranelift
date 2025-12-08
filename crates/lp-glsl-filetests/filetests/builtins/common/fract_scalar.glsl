// test compile
// test run

bool main() {
    float result = fract(3.75);  // 3.75 - floor(3.75) = 3.75 - 3.0 = 0.75
    return result > 0.74 && result < 0.76;
}

// function u0:0() -> i8 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.e00000p1
//     v1 = floor v0  ; v0 = 0x1.e00000p1
//     v2 = fsub v0, v1  ; v0 = 0x1.e00000p1
//     v3 = f32const 0x1.7ae148p-1
//     v4 = fcmp gt v2, v3  ; v3 = 0x1.7ae148p-1
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v4, v5, v6  ; v5 = 1, v6 = 0
//     v8 = f32const 0x1.851eb8p-1
//     v9 = fcmp lt v2, v8  ; v8 = 0x1.851eb8p-1
//     v10 = iconst.i8 1
//     v11 = iconst.i8 0
//     v12 = select v9, v10, v11  ; v10 = 1, v11 = 0
//     v13 = iconst.i8 0
//     v14 = iconst.i8 1
//     v15 = icmp ne v7, v13  ; v13 = 0
//     v16 = icmp ne v12, v13  ; v13 = 0
//     v17 = select v16, v14, v13  ; v14 = 1, v13 = 0
//     v18 = select v15, v17, v13  ; v13 = 0
//     return v18
//
// block1:
//     v19 = iconst.i8 0
//     return v19  ; v19 = 0
// }
// run: == true
