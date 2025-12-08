// test compile

int main() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    vec4 halved = v / 2.0;  // (5.0, 10.0, 15.0, 20.0)
    return 1;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v0 = f32const 0x1.400000p3
//     v1 = f32const 0x1.400000p4
//     v2 = f32const 0x1.e00000p4
//     v3 = f32const 0x1.400000p5
//     v4 = f32const 0x1.000000p1
//     v5 = fdiv v0, v4  ; v0 = 0x1.400000p3, v4 = 0x1.000000p1
//     v6 = fdiv v1, v4  ; v1 = 0x1.400000p4, v4 = 0x1.000000p1
//     v7 = fdiv v2, v4  ; v2 = 0x1.e00000p4, v4 = 0x1.000000p1
//     v8 = fdiv v3, v4  ; v3 = 0x1.400000p5, v4 = 0x1.000000p1
//     v9 = iconst.i32 1
//     return v9  ; v9 = 1
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }
