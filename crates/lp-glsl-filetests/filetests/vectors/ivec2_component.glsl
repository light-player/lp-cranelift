// test compile

// target riscv32.fixed32
int main() {
    ivec2 v = ivec2(10, 20);
    int x = v.x;
    int y = v.y;
    return x + y;  // 30
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 10
//     v1 = iconst.i32 20
//     v2 = iadd v0, v1  ; v0 = 10, v1 = 20
//     return v2
//
// block1:
//     v3 = iconst.i32 0
//     return v3  ; v3 = 0
// }
