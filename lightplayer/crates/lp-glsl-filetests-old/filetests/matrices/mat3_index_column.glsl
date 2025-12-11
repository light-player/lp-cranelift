// Test: mat3 column indexing
// Spec: operators.adoc:1019-1098 - Matrix indexing
vec3 main() {
    mat3 m = mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0);
    return m[1]; // Second column
}
// run: == vec3(4.0, 5.0, 6.0)




