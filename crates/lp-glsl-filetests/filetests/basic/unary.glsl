test compile
test run

int main() {
    int x = 10;
    return -x;
}

; CHECK: iconst.i32 10
; CHECK: ineg
; run: == -10

