// test compile
// test fixed32

int main() {
    float a = 2.5;
    float b = 3.5;
    if (a < b) {
        return 1;
    }
    return 0;
}

// function u0:0() -> i32 apple_aarch64 {
// block0:
//     v9 = iconst.i32 0x0002_8000
//     v10 = iconst.i32 0x0003_8000
//     v11 = icmp slt v9, v10  ; v9 = 0x0002_8000, v10 = 0x0003_8000
//     v3 = iconst.i8 1
//     v4 = iconst.i8 0
//     v5 = select v11, v3, v4  ; v3 = 1, v4 = 0
//     brif v5, block1, block2
//
// block1:
//     v6 = iconst.i32 1
//     return v6  ; v6 = 1
//
// block3:
//     jump block2
//
// block2:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
//
// block4:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
// }
