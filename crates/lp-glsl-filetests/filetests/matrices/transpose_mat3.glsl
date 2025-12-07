// Test: transpose mat3
// Spec: builtinfunctions.adoc:1538-1687
mat3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return transpose(m);
}
// run: == mat3(1.0, 4.0, 7.0, 2.0, 5.0, 8.0, 3.0, 6.0, 9.0)

