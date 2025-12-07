// test compile
// test run

bool main() {
    float result = step(5.0, 10.0);  // x(10.0) >= edge(5.0), returns 1.0
    return result > 0.99;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = f32const 0x1.400000p3
//     v2 = f32const 0.0
//     v3 = f32const 0x1.000000p0
//     v4 = fcmp lt v1, v0  ; v1 = 0x1.400000p3, v0 = 0x1.400000p2
//     v5 = select v4, v2, v3  ; v2 = 0.0, v3 = 0x1.000000p0
//     v6 = f32const 0x1.fae148p-1
//     v7 = fcmp gt v5, v6  ; v6 = 0x1.fae148p-1
//     v8 = iconst.i8 1
//     v9 = iconst.i8 0
//     v10 = select v7, v8, v9  ; v8 = 1, v9 = 0
//     return v10
//
// block1:
//     v11 = iconst.i8 0
//     return v11  ; v11 = 0
// }
// run: == true
