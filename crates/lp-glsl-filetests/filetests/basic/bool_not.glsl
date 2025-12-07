// test compile
// test run

bool main() {
    bool t = true;
    return !t;
}

// function u0:0() -> i8 system_v {
// block0:
//     v0 = iconst.i8 1
//     v1 = iconst.i8 0
//     v2 = icmp eq v0, v1  ; v0 = 1, v1 = 0
//     return v2
//
// block1:
//     v3 = iconst.i8 0
//     return v3  ; v3 = 0
// }
// run: == false
