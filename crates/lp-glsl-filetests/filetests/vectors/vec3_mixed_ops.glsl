// test compile

int main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(2.0, 2.0, 2.0);
    vec3 result = (a + b) * 2.0;  // ((3.0, 4.0, 5.0) * 2.0) = (6.0, 8.0, 10.0)
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p1
//     v4 = f32const 0x1.000000p1
//     v5 = f32const 0x1.000000p1
//     v6 = fadd v0, v3  ; v0 = 0x1.000000p0, v3 = 0x1.000000p1
//     v7 = fadd v1, v4  ; v1 = 0x1.000000p1, v4 = 0x1.000000p1
//     v8 = fadd v2, v5  ; v2 = 0x1.800000p1, v5 = 0x1.000000p1
//     v9 = f32const 0x1.000000p1
//     v10 = fmul v6, v9  ; v9 = 0x1.000000p1
//     v11 = fmul v7, v9  ; v9 = 0x1.000000p1
//     v12 = fmul v8, v9  ; v9 = 0x1.000000p1
//     v13 = iconst.i32 1
//     return v13  ; v13 = 1
//
// block1:
//     v14 = iconst.i32 0
//     return v14  ; v14 = 0
// }


