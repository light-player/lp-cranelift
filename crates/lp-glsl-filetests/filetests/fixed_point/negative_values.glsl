// test compile
// test fixed32

float main() {
    float a = -5.5;
    float b = 2.25;
    return a + b;
}

// function u0:0() -> i32 system_v {
// block0:
//     v5 = iconst.i32 0x0005_8000
//     v6 = ineg v5  ; v5 = 0x0005_8000
//     v7 = iconst.i32 0x0002_4000
//     v8 = iadd v6, v7  ; v7 = 0x0002_4000
//     return v8
//
// block1:
//     v9 = iconst.i32 0
//     return v9  ; v9 = 0
// }
