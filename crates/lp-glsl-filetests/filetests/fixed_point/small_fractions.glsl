// test compile
// test fixed32

float main() {
    float a = 0.0000152587890625;
    return a + a;
}

// function u0:0() -> i32 system_v {
// block0:
//     v3 = iconst.i32 1
//     v4 = iadd v3, v3  ; v3 = 1, v3 = 1
//     return v4
//
// block1:
//     v5 = iconst.i32 0
//     return v5  ; v5 = 0
// }
