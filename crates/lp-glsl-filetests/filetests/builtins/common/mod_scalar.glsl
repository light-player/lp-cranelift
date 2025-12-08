// test compile
// test run

bool main() {
    float result = mod(7.0, 3.0);  // 7 - 3*floor(7/3) = 7 - 3*2 = 1.0
    return result > 0.99 && result < 1.01;
}

// function u0:0() -> i8 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.c00000p2
//     v1 = f32const 0x1.800000p1
//     v2 = fdiv v0, v1  ; v0 = 0x1.c00000p2, v1 = 0x1.800000p1
//     v3 = floor v2
//     v4 = fmul v1, v3  ; v1 = 0x1.800000p1
//     v5 = fsub v0, v4  ; v0 = 0x1.c00000p2
//     v6 = f32const 0x1.fae148p-1
//     v7 = fcmp gt v5, v6  ; v6 = 0x1.fae148p-1
//     v8 = iconst.i8 1
//     v9 = iconst.i8 0
//     v10 = select v7, v8, v9  ; v8 = 1, v9 = 0
//     v11 = f32const 0x1.028f5cp0
//     v12 = fcmp lt v5, v11  ; v11 = 0x1.028f5cp0
//     v13 = iconst.i8 1
//     v14 = iconst.i8 0
//     v15 = select v12, v13, v14  ; v13 = 1, v14 = 0
//     v16 = iconst.i8 0
//     v17 = iconst.i8 1
//     v18 = icmp ne v10, v16  ; v16 = 0
//     v19 = icmp ne v15, v16  ; v16 = 0
//     v20 = select v19, v17, v16  ; v17 = 1, v16 = 0
//     v21 = select v18, v20, v16  ; v16 = 0
//     return v21
//
// block1:
//     v22 = iconst.i8 0
//     return v22  ; v22 = 0
// }
// run: == true
