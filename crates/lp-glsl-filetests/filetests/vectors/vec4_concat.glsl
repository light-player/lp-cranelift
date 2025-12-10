// test compile

// target riscv32.fixed32
int main() {
    vec2 xy = vec2(1.0, 2.0);
    vec4 v = vec4(xy, 3.0, 4.0);  // Concatenation
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v6 = iconst.i32 0x0001_0000
//     v7 = iconst.i32 0x0002_0000
//     v8 = iconst.i32 0x0003_0000
//     v9 = iconst.i32 0x0004_0000
//     v4 = iconst.i32 1
//     return v4  ; v4 = 1
//
// block1:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
