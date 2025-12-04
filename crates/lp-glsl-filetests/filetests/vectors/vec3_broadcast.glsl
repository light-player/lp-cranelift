// test compile

int main() {
    vec3 v = vec3(5.0);  // All components = 5.0
    return 1;
}

// function u0:0() -> i32 fast {
// block0:
//     v0 = f32const 0x1.400000p2
//     v1 = iconst.i32 1
//     return v1  ; v1 = 1
//
// block1:
//     v2 = iconst.i32 0
//     return v2  ; v2 = 0
// }
