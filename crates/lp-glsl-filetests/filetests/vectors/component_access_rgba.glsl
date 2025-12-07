// test compile

float main() {
    vec4 color = vec4(0.5, 0.6, 0.7, 1.0);
    float red = color.r;
    float alpha = color.a;
    return red + alpha;  // 1.5
}

// function u0:0() -> f32 system_v {
// block0:
//     v0 = f32const 0x1.000000p-1
//     v1 = f32const 0x1.333334p-1
//     v2 = f32const 0x1.666666p-1
//     v3 = f32const 0x1.000000p0
//     v4 = fadd v0, v3  ; v0 = 0x1.000000p-1, v3 = 0x1.000000p0
//     return v4
//
// block1:
//     v5 = f32const 0.0
//     return v5  ; v5 = 0.0
// }
