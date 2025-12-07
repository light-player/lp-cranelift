// Test: determinant mat3
// Spec: builtinfunctions.adoc:1538-1687
float main() {
    mat3 m = mat3(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
    return determinant(m);
}
// run: == 1.0  // Identity matrix has determinant 1

