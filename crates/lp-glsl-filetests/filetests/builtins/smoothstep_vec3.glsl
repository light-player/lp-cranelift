// test compile
// test run

bool main() {
    vec3 edge0 = vec3(0.0, 0.0, 0.0);
    vec3 edge1 = vec3(10.0, 10.0, 10.0);
    vec3 x = vec3(5.0, 2.5, 7.5);
    vec3 result = smoothstep(edge0, edge1, x);
    // Expected: smoothstep(0,10,5)≈0.5, smoothstep(0,10,2.5)≈0.15625, smoothstep(0,10,7.5)≈0.84375
    // Sum ≈ 1.5
    float sum = result.x + result.y + result.z;
    return sum > 1.45 && sum < 1.55;
}

// run: == true

// Verify component-wise smoothstep calculation
// CHECK-LABEL: function u0:0
// CHECK: fsub  ; x[0] - edge0[0]
// CHECK: fsub  ; edge1[0] - edge0[0]
// CHECK: fdiv  ; t_raw[0]
// CHECK: fmax  ; clamp step 1
// CHECK: fmin  ; clamp step 2
// CHECK: fmul  ; t²[0]
// CHECK: fmul  ; 2*t[0]
// CHECK: fsub  ; 3-2*t[0]
// CHECK: fmul  ; final result[0]

