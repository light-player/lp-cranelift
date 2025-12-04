// test compile
// test run

float main() {
    float sum = 0.0;
    for (int i = 0; i < 3; i = i + 1) {
        sum = sum + 1.5;
    }
    return sum;
}

// function u0:0() -> f32 fast {
// block0:
//     v0 = f32const 0.0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0.0
//
// block1(v2: i32, v5: f32):
//     v3 = iconst.i32 3
//     v4 = icmp slt v2, v3  ; v3 = 3
//     brif v4, block2, block4
//
// block2:
//     v6 = f32const 0x1.800000p0
//     v7 = fadd.f32 v5, v6  ; v6 = 0x1.800000p0
//     jump block3
//
// block3:
//     v8 = iconst.i32 1
//     v9 = iadd.i32 v2, v8  ; v8 = 1
//     jump block1(v9, v7)
//
// block4:
//     return v5
//
// block5:
//     v10 = f32const 0.0
//     return v10  ; v10 = 0.0
// }
// run: ~= 0 (tolerance: 0.01)
