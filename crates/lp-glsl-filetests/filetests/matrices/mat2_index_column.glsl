// Test: mat2 column indexing
// Spec: operators.adoc:1019-1098 - Matrix indexing
vec2 main() {
    mat2 m = mat2(1.0, 2.0, 3.0, 4.0);
    return m[0]; // First column
}
// run: == vec2(1.0, 2.0)



