// test compile

// target riscv32.fixed32
int main() {
    vec3 v = vec3(5.0);  // All components = 5.0
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0005_0000
//     v1 = iconst.i32 1
//     return v1  ; v1 = 1
//
// block1:
//     v2 = iconst.i32 0
//     return v2  ; v2 = 0
// }
