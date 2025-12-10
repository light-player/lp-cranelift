// test compile

// target riscv32.fixed32
int main() {
    ivec3 i = ivec3(1, 2, 3);
    vec3 v = vec3(i);  // Type conversion
    return 1;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 1
//     v1 = iconst.i32 2
//     v2 = iconst.i32 3
//     v3 = iconst.i32 16
//     v4 = ishl v0, v3  ; v0 = 1, v3 = 16
//     v5 = iconst.i32 16
//     v6 = ishl v1, v5  ; v1 = 2, v5 = 16
//     v7 = iconst.i32 16
//     v8 = ishl v2, v7  ; v2 = 3, v7 = 16
//     v9 = iconst.i32 1
//     return v9  ; v9 = 1
//
// block1:
//     v10 = iconst.i32 0
//     return v10  ; v10 = 0
// }
