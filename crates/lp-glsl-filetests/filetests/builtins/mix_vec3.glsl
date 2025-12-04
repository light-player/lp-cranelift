// test compile
// test run

bool main() {
    vec3 a = vec3(0.0, 0.0, 0.0);
    vec3 b = vec3(10.0, 20.0, 30.0);
    vec3 result = mix(a, b, vec3(0.5, 0.5, 0.5));  // (5.0, 10.0, 15.0)
    // Validate: x=5, y=10, z=15, sum=30
    float sum = result.x + result.y + result.z;
    return sum > 29.99 && sum < 30.01;
}

// run: == true

// Verify component-wise mix: each component does x*(1-a) + y*a
// CHECK-LABEL: function u0:0
// CHECK-DAG: f32const 0.0
// CHECK-DAG: f32const 0x1.400000p3  ; 10.0
// CHECK-DAG: f32const 0x1.800000p4  ; 20.0
// CHECK-DAG: f32const 0x1.e00000p4  ; 30.0
// CHECK-DAG: f32const 0x1.000000p-1 ; 0.5
// CHECK: f32const 0x1.000000p0      ; 1.0
// CHECK: fsub  ; (1 - a[0])
// CHECK: fmul  ; a[0] * (1-a[0])
// CHECK: fmul  ; b[0] * a[0]
// CHECK: fadd  ; mix result[0]

