// test compile
// test run

bool main() {
    vec3 result = smoothstep(0.0, 10.0, vec3(0.0, 5.0, 10.0));
    // Expected: smoothstep(0,10,0)=0, smoothstep(0,10,5)=0.5, smoothstep(0,10,10)=1.0
    // Sum = 1.5
    float sum = result.x + result.y + result.z;
    return sum > 1.49 && sum < 1.51;
}

// run: == true

// Verify scalar edge broadcast with smoothstep(float, float, vec3)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0.0  ; edge0 (scalar)
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p3  ; edge1 = 10.0 (scalar)
// CHECK: fsub  ; x[0] - edge0
// CHECK: fsub  ; edge1 - edge0 (computed once)
// CHECK: fdiv  ; t[0]
// CHECK: fmax  ; clamp
// CHECK: fmin  ; clamp
// CHECK: fmul  ; t²
// CHECK: fmul  ; 2*t
// CHECK: fsub  ; 3-2*t
// CHECK: fmul  ; result[0]

