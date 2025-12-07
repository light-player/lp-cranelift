// Test: matrixCompMult component-wise multiply
// Spec: builtinfunctions.adoc:1538-1687
mat2 main() {
    mat2 a = mat2(1.0, 2.0, 3.0, 4.0);
    mat2 b = mat2(2.0, 2.0, 2.0, 2.0);
    return matrixCompMult(a, b);
}
// run: == mat2(2.0, 4.0, 6.0, 8.0)

