// Test: scalar × matrix (component-wise)
// Spec: operators.adoc:1019-1098 - Matrix arithmetic
mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return 2.0 * m;
}
// run: == mat2(2.0, 4.0, 6.0, 8.0)

