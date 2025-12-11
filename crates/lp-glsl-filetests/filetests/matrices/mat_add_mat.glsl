// Test: matrix + matrix (component-wise)
// Spec: operators.adoc:1019-1098 - Matrix arithmetic
mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(5.0, 6.0, 7.0, 8.0);
    return a + b;
}
// run: == mat2(6.0, 8.0, 10.0, 12.0)



