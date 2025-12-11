// Test: vec3 × mat3 (linear algebra)
// Spec: operators.adoc:1019-1098 - Vector-matrix multiply
vec3 main() {
    mat3 m = mat3(1.0);  // Identity
    vec3 v = vec3(2.0, 3.0, 4.0);
    return v * m;
}
// run: == vec3(2.0, 3.0, 4.0)



