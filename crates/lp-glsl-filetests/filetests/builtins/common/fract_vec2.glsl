// test compile
// test run

bool main() {
    vec2 result = fract(vec2(3.75, 5.25));  // (0.75, 0.25)
    // Validate: sum = 0.75 + 0.25 = 1.0
    float sum = result.x + result.y;
    return sum > 0.99 && sum < 1.01;
}

// function u0:0() -> i8 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.e00000p1
//     v1 = f32const 0x1.500000p2
//     v2 = floor v0  ; v0 = 0x1.e00000p1
//     v3 = fsub v0, v2  ; v0 = 0x1.e00000p1
//     v4 = floor v1  ; v1 = 0x1.500000p2
//     v5 = fsub v1, v4  ; v1 = 0x1.500000p2
//     v6 = fadd v3, v5
//     v7 = f32const 0x1.fae148p-1
//     v8 = fcmp gt v6, v7  ; v7 = 0x1.fae148p-1
//     v9 = iconst.i8 1
//     v10 = iconst.i8 0
//     v11 = select v8, v9, v10  ; v9 = 1, v10 = 0
//     v12 = f32const 0x1.028f5cp0
//     v13 = fcmp lt v6, v12  ; v12 = 0x1.028f5cp0
//     v14 = iconst.i8 1
//     v15 = iconst.i8 0
//     v16 = select v13, v14, v15  ; v14 = 1, v15 = 0
//     v17 = iconst.i8 0
//     v18 = iconst.i8 1
//     v19 = icmp ne v11, v17  ; v17 = 0
//     v20 = icmp ne v16, v17  ; v17 = 0
//     v21 = select v20, v18, v17  ; v18 = 1, v17 = 0
//     v22 = select v19, v21, v17  ; v17 = 0
//     return v22
//
// block1:
//     v23 = iconst.i8 0
//     return v23  ; v23 = 0
// }
// run: == true
