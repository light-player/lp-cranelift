// Test: mat3 × vec3 (linear algebra)
// Spec: operators.adoc:1019-1098 - Matrix-vector multiply
vec3 main() {
    mat3 m = mat3(1.0);  // Identity
    vec3 v = vec3(2.0, 3.0, 4.0);
    return m * v;
}
// run: == vec3(2.0, 3.0, 4.0)

