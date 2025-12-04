// test compile
// test fixed32

float main() {
    float a = 5.5;
    float b = 2.25;
    return a - b;
}

// function u0:0() -> i32 fast {
// block0:
//     v4 = iconst.i32 0x0005_8000
//     v5 = iconst.i32 0x0002_4000
//     v6 = isub v4, v5  ; v4 = 0x0005_8000, v5 = 0x0002_4000
//     return v6
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
