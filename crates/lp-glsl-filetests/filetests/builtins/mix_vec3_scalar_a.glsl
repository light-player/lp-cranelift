// test compile
// test run

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, 0.25);  // 0*0.75 + vec3(10,20,30)*0.25 = (2.5, 5.0, 7.5)
    // Validate: sum = 2.5 + 5.0 + 7.5 = 15.0
    float sum = result.x + result.y + result.z;
    return sum > 14.99 && sum < 15.01;
}

// run: == true

// Verify scalar broadcast: mix(vec3, vec3, float) reuses (1-a) for all components
// CHECK-LABEL: function u0:0
// CHECK: f32const 0x1.000000p-2  ; 0.25
// CHECK: f32const 0x1.000000p0   ; 1.0
// CHECK: fsub  ; (1 - 0.25) = 0.75
// CHECK: fmul  ; a[0] * 0.75
// CHECK: fmul  ; b[0] * 0.25
// CHECK: fadd  ; result[0]
// CHECK: fmul  ; a[1] * 0.75
// CHECK: fmul  ; b[1] * 0.25
// CHECK: fadd  ; result[1]

