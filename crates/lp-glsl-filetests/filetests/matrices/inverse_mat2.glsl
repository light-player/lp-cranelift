// Test: inverse mat2
// Spec: builtinfunctions.adoc:1538-1687
mat2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return inverse(m);
}
// run: == mat2(-2.0, 1.0, 1.5, -0.5)  // Inverse of [1 3; 2 4] = [-2 1.5; 1 -0.5]

