// test compile
// test run

float main() {
    float x = 10;  // int 10 → float conversion
    return x;
}

// function u0:0() -> f32 fast {
// block0:
//     v0 = iconst.i32 10
//     v1 = fcvt_from_sint.f32 v0  ; v0 = 10
//     return v1
//
// block1:
//     v2 = f32const 0.0
//     return v2  ; v2 = 0.0
// }
// run: ~= 0 (tolerance: 0.01)

