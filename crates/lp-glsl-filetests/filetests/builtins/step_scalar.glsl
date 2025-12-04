// test compile
// test run

bool main() {
    float result = step(5.0, 10.0);  // x(10.0) >= edge(5.0), returns 1.0
    return result > 0.99;
}

// run: == true

// Verify step(edge, x) = (x < edge) ? 0.0 : 1.0
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p2  ; 10.0
// CHECK: v{{[0-9]+}} = f32const 0x1.400000p2  ; 5.0
// CHECK: v{{[0-9]+}} = fcmp lt
// CHECK: v{{[0-9]+}} = f32const 0.0
// CHECK: v{{[0-9]+}} = f32const 0x1.000000p0  ; 1.0
// CHECK: v{{[0-9]+}} = select

