// Test: outerProduct vec3 × vec3 → mat3
// Spec: builtinfunctions.adoc:1538-1687
mat3 main() {
    vec3 u = vec3(1.0, 2.0, 3.0);
    vec3 v = vec3(4.0, 5.0, 6.0);
    return outerProduct(u, v);
}
// run: == mat3(4.0, 8.0, 12.0, 5.0, 10.0, 15.0, 6.0, 12.0, 18.0)

