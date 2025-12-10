// test compile  
// test fixed32

float main() {
    float a = 32767.0;
    return a - 32766.0;
}

// function u0:0() -> i32 system_v {
// block0:
//     v4 = iconst.i32 0x7fff_0000
//     v5 = iconst.i32 0x7ffe_0000
//     v6 = isub v4, v5  ; v4 = 0x7fff_0000, v5 = 0x7ffe_0000
//     return v6
//
// block1:
//     v7 = iconst.i32 0
//     return v7  ; v7 = 0
// }
