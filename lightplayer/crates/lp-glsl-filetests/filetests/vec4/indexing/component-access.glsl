// test run
// target riscv32.fixed32

// ============================================================================
// Component access: v.x, v.y, v.z, v.w, v.r, v.g, v.b, v.a, v.s, v.t, v.p, v.q
// ============================================================================

float test_vec4_component_access_xyzw() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Access using xyzw names
    return v.x + v.y + v.z + v.w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_component_access_xyzw() ~= 10.0

float test_vec4_component_access_rgba() {
    vec4 v = vec4(10.0, 20.0, 30.0, 40.0);
    // Access using rgba names (synonyms for xyzw)
    return v.r + v.g + v.b + v.a;
    // Should be 10.0 + 20.0 + 30.0 + 40.0 = 100.0
}

// run: test_vec4_component_access_rgba() ~= 100.0

float test_vec4_component_access_stpq() {
    vec4 v = vec4(100.0, 200.0, 300.0, 400.0);
    // Access using stpq names (synonyms for xyzw)
    return v.s + v.t + v.p + v.q;
    // Should be 100.0 + 200.0 + 300.0 + 400.0 = 1000.0
}

// run: test_vec4_component_access_stpq() ~= 1000.0

float test_vec4_component_access_mixed() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Mix different naming conventions
    float sum = v.x + v.g + v.z + v.a;
    // x=1.0, g=2.0, z=3.0, a=4.0
    return sum;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_component_access_mixed() ~= 10.0

float test_vec4_component_access_verify_synonyms() {
    vec4 v = vec4(5.0, 10.0, 15.0, 20.0);
    // Verify that x=r=s, y=g=t, z=b=p, w=a=q
    float sum = 0.0;
    if (v.x == v.r && v.r == v.s) sum = sum + 1.0;
    if (v.y == v.g && v.g == v.t) sum = sum + 1.0;
    if (v.z == v.b && v.b == v.p) sum = sum + 1.0;
    if (v.w == v.a && v.a == v.q) sum = sum + 1.0;
    return sum;
    // Should be 4.0 (all synonyms match)
}

// run: test_vec4_component_access_verify_synonyms() ~= 4.0

float test_vec4_component_access_read() {
    vec4 v = vec4(1.0, 2.0, 3.0, 4.0);
    // Read individual components
    float x = v.x;
    float y = v.y;
    float z = v.z;
    float w = v.w;
    return x + y + z + w;
    // Should be 1.0 + 2.0 + 3.0 + 4.0 = 10.0
}

// run: test_vec4_component_access_read() ~= 10.0

