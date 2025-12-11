// test compile
// test run
// target riscv32.fixed32

int main() {
    float a = 2.5;
    float b = 1.5;
    if (a > b) {
        return 1;
    }
    return 0;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0002_8000
//     v1 = iconst.i32 0x0001_8000
//     v2 = icmp sgt v0, v1  ; v0 = 0x0002_8000, v1 = 0x0001_8000
//     v3 = sextend.i32 v2
//     v4 = iconst.i8 1
//     v5 = iconst.i8 0
//     v6 = select v3, v4, v5  ; v4 = 1, v5 = 0
//     brif v6, block1, block3
//
// block1:
//     v7 = iconst.i32 1
//     return v7  ; v7 = 1
//
// block2:
//     jump block3
//
// block3:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
//
// block4:
//     v9 = iconst.i32 0
//     return v9  ; v9 = 0
// }
// run: == 1
