// test compile
// test run
// target riscv32

int main() {
    int x = 5;
    if (x > 0) {
        return 42;
    }
    return 0;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 5
//     v1 = iconst.i32 0
//     v2 = icmp sgt v0, v1  ; v0 = 5, v1 = 0
//     v3 = iconst.i8 1
//     v4 = iconst.i8 0
//     v5 = select v2, v3, v4  ; v3 = 1, v4 = 0
//     brif v5, block1, block2
//
// block1:
//     v6 = iconst.i32 42
//     return v6  ; v6 = 42
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
// run: == 42
