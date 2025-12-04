// test compile
// test run

int main() {
    int a = sign(-5);  // -1
    int b = sign(0);   // 0
    int c = sign(5);   // 1
    // Validate sum: -1 + 0 + 1 = 0
    return a + b + c;
}

// run: == 0

// Verify sign(x) for integers: x>0 ? 1 : (x<0 ? -1 : 0)
// CHECK-LABEL: function u0:0
// CHECK: v{{[0-9]+}} = iconst.i32 -5
// CHECK: v{{[0-9]+}} = iconst.i32 0
// CHECK: v{{[0-9]+}} = icmp sgt  ; x > 0
// CHECK: v{{[0-9]+}} = icmp slt  ; x < 0
// CHECK: v{{[0-9]+}} = iconst.i32 1
// CHECK: v{{[0-9]+}} = select    ; x>0 ? 1 : 0
// CHECK: v{{[0-9]+}} = iconst.i32 -1
// CHECK: v{{[0-9]+}} = select    ; x<0 ? -1 : prev

