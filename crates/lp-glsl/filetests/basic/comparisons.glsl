test compile
test run

bool main() {
    int a = 10;
    int b = 20;
    return a < b;
}

; CHECK: iconst.i32 10
; CHECK: iconst.i32 20
; CHECK: icmp slt
; run: == true

