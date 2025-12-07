// test compile
// test run

bool main() {
    int a = 10;
    int b = 20;
    return a < b;
}

// function u0:0() -> i8 fast {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 20
//     v2 = icmp slt v0, v1  ; v0 = 10, v1 = 20
//     v3 = iconst.i8 1
//     v4 = iconst.i8 0
//     v5 = select v2, v3, v4  ; v3 = 1, v4 = 0
//     return v5
//
// block1:
//     v6 = iconst.i8 0
//     return v6  ; v6 = 0
// }
// run: == true
