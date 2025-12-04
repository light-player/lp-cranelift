// test compile

int main() {
    ivec3 i = ivec3(1, 2, 3);
    vec3 v = vec3(i);  // Type conversion
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = iconst.i32 1
//     v1 = iconst.i32 2
//     v2 = iconst.i32 3
//     v3 = fcvt_from_sint.f32 v0  ; v0 = 1
//     v4 = fcvt_from_sint.f32 v1  ; v1 = 2
//     v5 = fcvt_from_sint.f32 v2  ; v2 = 3
//     v6 = iconst.i32 1
//     return v6  ; v6 = 1
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
