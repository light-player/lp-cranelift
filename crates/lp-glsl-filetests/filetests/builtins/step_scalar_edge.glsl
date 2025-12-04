// test compile
// test run

bool main() {
    vec3 result = step(5.0, vec3(3.0, 5.0, 7.0));  // (0.0, 1.0, 1.0)
    // Validate: sum = 2.0
    float sum = result.x + result.y + result.z;
    return sum > 1.99 && sum < 2.01;
}

// run: == true

// Verify scalar edge broadcast: step(float, vec3)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p2  ; 5.0 (edge)
// CHECK: v{{[0-9]+}} = fcmp lt  ; x[0] < edge
// CHECK: v{{[0-9]+}} = f32const 0.0
// CHECK: v{{[0-9]+}} = f32const 0x1.000000p0  ; 1.0
// CHECK: v{{[0-9]+}} = select
// CHECK: v{{[0-9]+}} = fcmp lt  ; x[1] < edge (reuses edge)
// CHECK: v{{[0-9]+}} = select
// CHECK: v{{[0-9]+}} = fcmp lt  ; x[2] < edge (reuses edge)
// CHECK: v{{[0-9]+}} = select

