// test compile

int main() {
    ivec2 a = ivec2(10, 20);
    ivec2 b = ivec2(3, 7);
    ivec2 c = a - b;  // (7, 13)
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 20
//     v2 = iconst.i32 3
//     v3 = iconst.i32 7
//     v4 = isub v0, v2  ; v0 = 10, v2 = 3
//     v5 = isub v1, v3  ; v1 = 20, v3 = 7
//     v6 = iconst.i32 1
//     return v6  ; v6 = 1
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
