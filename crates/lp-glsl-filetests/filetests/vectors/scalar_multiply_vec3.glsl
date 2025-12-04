// test compile

int main() {
    vec3 scaled = 3.0 * vec3(1.0, 2.0, 3.0);  // (3.0, 6.0, 9.0)
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = f32const 0x1.800000p1
//     v1 = f32const 0x1.000000p0
//     v2 = f32const 0x1.000000p1
//     v3 = f32const 0x1.800000p1
//     v4 = fmul v0, v1  ; v0 = 0x1.800000p1, v1 = 0x1.000000p0
//     v5 = fmul v0, v2  ; v0 = 0x1.800000p1, v2 = 0x1.000000p1
//     v6 = fmul v0, v3  ; v0 = 0x1.800000p1, v3 = 0x1.800000p1
//     v7 = iconst.i32 1
//     return v7  ; v7 = 1
//
// block1:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
// }
