// test compile

int main() {
    vec2 xy = vec2(1.0, 2.0);
    vec4 v = vec4(xy, 3.0, 4.0);  // Concatenation
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = f32const 0x1.000000p0
//     v1 = f32const 0x1.000000p1
//     v2 = f32const 0x1.800000p1
//     v3 = f32const 0x1.000000p2
//     v4 = iconst.i32 1
//     return v4  ; v4 = 1
//
// block1:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
