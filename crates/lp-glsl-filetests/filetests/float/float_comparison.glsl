// test compile
// test run

int main() {
    float a = 2.5;
    float b = 1.5;
    if (a > b) {
        return 1;
    }
    return 0;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = f32const 0x1.400000p1
//     v1 = f32const 0x1.800000p0
//     v2 = fcmp gt v0, v1  ; v0 = 0x1.400000p1, v1 = 0x1.800000p0
//     brif v2, block1, block2
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
// run: == 1
