// test compile
// test fixed64

float main() {
    return 3.14159;
}

// function u0:0() -> i64 fast {
// block0:
//     v2 = iconst.i64 0x0003_243f_4000
//     return v2  ; v2 = 0x0003_243f_4000
//
// block1:
//     v3 = iconst.i64 0
//     return v3  ; v3 = 0
// }
