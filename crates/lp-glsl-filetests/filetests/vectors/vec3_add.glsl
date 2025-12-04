// test compile
// test run

int main() {
    vec3 a = vec3(1.0, 2.0, 3.0);
    vec3 b = vec3(4.0, 5.0, 6.0);
    vec3 c = a + b;  // (5.0, 7.0, 9.0)
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = f32const 0x1.400000p2
//     v5 = f32const 0x1.800000p2
//     v6 = fadd v0, v3  ; v0 = 0x1.000000p0, v3 = 0x1.000000p2
//     v7 = fadd v1, v4  ; v1 = 0x1.000000p1, v4 = 0x1.400000p2
//     v8 = fadd v2, v5  ; v2 = 0x1.800000p1, v5 = 0x1.800000p2
//     v9 = iconst.i32 1
//     return v9  ; v9 = 1
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }
// run: == 1
