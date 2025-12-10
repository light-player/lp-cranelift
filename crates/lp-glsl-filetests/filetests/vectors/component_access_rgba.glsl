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
//     v6 = iconst.i32 0x8000
//     v7 = iconst.i32 0x999a
//     v8 = iconst.i32 0xb333
//     v9 = iconst.i32 0x0001_0000
//     v10 = iadd v6, v9  ; v6 = 0x8000, v9 = 0x0001_0000
//     return v10
//
// block1:
//     v11 = iconst.i32 0
//     return v11  ; v11 = 0
// }

