// test compile
// test run

bool main() {
    // smoothstep(0, 10, 5): t = 0.5, result = t²(3-2t) = 0.25*2 = 0.5
    float result = smoothstep(0.0, 10.0, 5.0);
    return result > 0.49 && result < 0.51;
}

// run: == true

// Verify smoothstep formula: t = clamp((x-edge0)/(edge1-edge0), 0, 1); return t²(3-2t)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p2  ; 5.0 (x)
// CHECK: v{{[0-9]+}} = f32const 0.0           ; edge0
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p3  ; 10.0 (edge1)
// CHECK: v{{[0-9]+}} = fsub  ; x - edge0
// CHECK: v{{[0-9]+}} = fsub  ; edge1 - edge0
// CHECK: v{{[0-9]+}} = fdiv  ; t_raw = (x-edge0)/(edge1-edge0)
// CHECK: v{{[0-9]+}} = fmax  ; max(t_raw, 0)
// CHECK: v{{[0-9]+}} = fmin  ; t = min(max(t_raw, 0), 1)
// CHECK: v{{[0-9]+}} = fmul  ; t²
// CHECK: v{{[0-9]+}} = f32const 0x1.000000p1  ; 2.0
// CHECK: v{{[0-9]+}} = fmul  ; 2*t
// CHECK: v{{[0-9]+}} = f32const 0x1.800000p1  ; 3.0
// CHECK: v{{[0-9]+}} = fsub  ; 3-2*t
// CHECK: v{{[0-9]+}} = fmul  ; t²*(3-2*t)

