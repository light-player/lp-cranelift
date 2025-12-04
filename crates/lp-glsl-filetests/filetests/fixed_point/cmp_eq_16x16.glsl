// test compile
// test fixed16

int main() {
    float a = 3.5;
    float b = 3.5;
    if (a == b) {
        return 1;
    }
    return 0;
}

// function u0:0() -> i32 fast {
// block0:
//     v6 = iconst.i32 0x0003_8000
//     v7 = iconst.i32 0x0003_8000
//     v8 = icmp eq v6, v7  ; v6 = 0x0003_8000, v7 = 0x0003_8000
//     brif v8, block1, block2
//
// block1:
//     v3 = iconst.i32 1
//     return v3  ; v3 = 1
//
// block3:
//     jump block2
//
// block2:
//     v4 = iconst.i32 0
//     return v4  ; v4 = 0
//
// block4:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
