// test compile
// test run
// target riscv32.fixed32

float main() {
    float sum = 0.0;
    for (int i = 0; i < 3; i = i + 1) {
        sum = sum + 1.5;
    }
    return sum;
}

// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0
//     v1 = iconst.i32 0
//     jump block1(v1, v0)  ; v1 = 0, v0 = 0
//
// block1(v2: i32, v3: i32):
//     v4 = iconst.i32 3
//     v5 = icmp slt v2, v4  ; v4 = 3
//     v6 = iconst.i8 1
//     v7 = iconst.i8 0
//     v8 = select v5, v6, v7  ; v6 = 1, v7 = 0
//     brif v8, block2, block4
//
// block2:
//     v9 = iconst.i32 0x0001_8000
//     v10 = iadd.i32 v3, v9  ; v9 = 0x0001_8000
//     jump block3
//
// block3:
//     v11 = iconst.i32 1
//     v12 = iadd.i32 v2, v11  ; v11 = 1
//     jump block1(v12, v10)
//
// block4:
//     return v3
//
// block5:
//     v13 = iconst.i32 0
//     return v13  ; v13 = 0
// }
// run: ~= 4.5 (tolerance: 0.01)
