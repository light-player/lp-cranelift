// test riscv32.fixed32
// test run

float main() {
    if (7.0 > 5.0) {
        return 1.0;
    } else {
        return 0.0;
    }
}

// Generated CLIF
// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.c00000p2
//     v1 = f32const 0x1.400000p2
//     v2 = fcmp gt v0, v1  ; v0 = 0x1.c00000p2, v1 = 0x1.400000p2
//     v3 = iconst.i8 1
//     v4 = iconst.i8 0
//     v5 = select v2, v3, v4  ; v3 = 1, v4 = 0
//     brif v5, block1, block3
//
// block1:
//     v6 = f32const 0x1.000000p0
//     return v6  ; v6 = 0x1.000000p0
//
// block4:
//     jump block2
//
// block3:
//     v7 = f32const 0.0
//     return v7  ; v7 = 0.0
//
// block5:
//     jump block2
//
// block2:
//     v8 = f32const 0.0
//     return v8  ; v8 = 0.0
// }
//
// Transformed CLIF
// function u0:0() -> i32 system_v {
// block0:
//     v0 = iconst.i32 0x0007_0000
//     v1 = iconst.i32 0x0005_0000
//     v2 = icmp sgt v0, v1  ; v0 = 0x0007_0000, v1 = 0x0005_0000
//     v3 = sextend.i32 v2
//     v4 = iconst.i8 1
//     v5 = iconst.i8 0
//     v6 = select v3, v4, v5  ; v4 = 1, v5 = 0
//     brif v6, block1, block3
//
// block1:
//     v7 = iconst.i32 0x0001_0000
//     return v7  ; v7 = 0x0001_0000
//
// block2:
//     jump block5
//
// block3:
//     v8 = iconst.i32 0
//     return v8  ; v8 = 0
//
// block4:
//     jump block5
//
// block5:
//     v9 = iconst.i32 0
//     return v9  ; v9 = 0
// }
// run: ≈ 1.0
