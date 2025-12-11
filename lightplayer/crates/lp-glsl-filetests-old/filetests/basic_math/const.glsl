// test run
// test fixed32

float main() {
    return 3.14159;
}

// run: ~= 3.14159

// function u0:0() -> i32 system_v {
// block0:
//     v2 = iconst.i32 0x0003_243f
//     return v2  ; v2 = 0x0003_243f
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
