// Test: mat3 construction from column vectors
// Spec: variables.adoc:72-97 - Matrix type definitions
mat3 main() {
    vec3 col0 = vec3(1.0, 2.0, 3.0);
    vec3 col1 = vec3(4.0, 5.0, 6.0);
    vec3 col2 = vec3(7.0, 8.0, 9.0);
    return mat3(col0, col1, col2);
}
// run: == mat3(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0)


