// Test: determinant mat2
// Spec: builtinfunctions.adoc:1538-1687
float main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return determinant(m);
}
// run: == -2.0  // 1*4 - 2*3 = 4 - 6 = -2

