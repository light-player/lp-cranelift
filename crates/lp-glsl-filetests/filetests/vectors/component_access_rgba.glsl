// test compile

// target riscv32.fixed32
float main() {
    vec4 color = vec4(0.5, 0.6, 0.7, 1.0);
    float red = color.r;
    float alpha = color.a;
    return red + alpha;  // 1.5
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x8000
//     v1 = iconst.i32 0x999a
//     v2 = iconst.i32 0xb333
//     v3 = iconst.i32 0x0001_0000
//     v4 = iadd v0, v3  ; v0 = 0x8000, v3 = 0x0001_0000
//     return v4
//
// block1:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }

