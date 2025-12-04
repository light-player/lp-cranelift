test compile
test run

int main() {
    int a = 5;
    int b = 3;
    int c = 2;
    return (a + b) * c - 4;
}

; CHECK: iadd
; CHECK: imul
; CHECK: isub
; run: == 12

