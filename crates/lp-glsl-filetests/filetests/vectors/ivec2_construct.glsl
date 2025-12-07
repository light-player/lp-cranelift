// test compile

int main() {
    ivec2 v = ivec2(10, 20);
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 20
//     v2 = iconst.i32 1
//     return v2  ; v2 = 1
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
