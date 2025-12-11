// Test: mat4 column indexing
// Spec: operators.adoc:1019-1098 - Matrix indexing
vec4 main() {
    mat4 m = mat4(1.0);
    return m[2]; // Third column (should be 0,0,1,0 for identity)
}
// run: == vec4(0.0, 0.0, 1.0, 0.0)



