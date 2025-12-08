// test compile
// test fixed32

float main() {
    float a = 2.5;
    float b = 1.5;
    return a + b;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v4 = iconst.i32 0x0002_8000
//     v5 = iconst.i32 0x0001_8000
//     v6 = iadd v4, v5  ; v4 = 0x0002_8000, v5 = 0x0001_8000
//     return v6
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
