// test compile
// test run
// target riscv32

float main() {
    float sum = 0.0;
    for (int i = 0; i < 3; i = i + 1) {
        sum = sum + 1.5;
    }
    return sum;
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0.0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0.0
//
// block1(v2: i32, v8: f32):
//     v3 = iconst.i32 3
//     v4 = icmp slt v2, v3  ; v3 = 3
//     v5 = iconst.i8 1
//     v6 = iconst.i8 0
//     v7 = select v4, v5, v6  ; v5 = 1, v6 = 0
//     brif v7, block2, block4
//
// block2:
//     v9 = f32const 0x1.800000p0
//     v10 = fadd.f32 v8, v9  ; v9 = 0x1.800000p0
//     jump block3
//
// block3:
//     v11 = iconst.i32 1
//     v12 = iadd.i32 v2, v11  ; v11 = 1
//     jump block1(v12, v10)
//
// block4:
//     return v8
//
// block5:
//     v13 = f32const 0.0
//     return v13  ; v13 = 0.0
// }
// run: ~= 4.5 (tolerance: 0.01)
