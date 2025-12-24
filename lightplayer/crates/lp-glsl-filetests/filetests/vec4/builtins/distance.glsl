// test run
// target riscv32.fixed32

// ============================================================================
// Distance: distance(vec4, vec4) - distance between two points
// distance(p0, p1) = length(p0 - p1)
// ============================================================================

float test_vec4_distance() {
    vec4 p0 = vec4(0.0, 0.0, 0.0, 0.0);
    vec4 p1 = vec4(1.0, 2.0, 2.0, 0.0);
    return distance(p0, p1);
    // Should be sqrt(1 + 4 + 4 + 0) = sqrt(9) = 3.0
}

// run: test_vec4_distance() ~= 3.0

float test_vec4_distance_same_point() {
    vec4 p0 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 p1 = vec4(1.0, 2.0, 3.0, 4.0);
    return distance(p0, p1);
    // Should be 0.0 (same point)
}

// run: test_vec4_distance_same_point() ~= 0.0

float test_vec4_distance_unit() {
    vec4 p0 = vec4(0.0, 0.0, 0.0, 0.0);
    vec4 p1 = vec4(1.0, 0.0, 0.0, 0.0);
    return distance(p0, p1);
    // Should be 1.0 (unit distance)
}

// run: test_vec4_distance_unit() ~= 1.0

float test_vec4_distance_all_components() {
    vec4 p0 = vec4(1.0, 1.0, 1.0, 1.0);
    vec4 p1 = vec4(2.0, 2.0, 2.0, 2.0);
    return distance(p0, p1);
    // Should be sqrt(1 + 1 + 1 + 1) = sqrt(4) = 2.0
}

// run: test_vec4_distance_all_components() ~= 2.0

float test_vec4_distance_negative() {
    vec4 p0 = vec4(0.0, 0.0, 0.0, 0.0);
    vec4 p1 = vec4(-3.0, -4.0, 0.0, 0.0);
    return distance(p0, p1);
    // Should be sqrt(9 + 16) = sqrt(25) = 5.0
}

// run: test_vec4_distance_negative() ~= 5.0

float test_vec4_distance_symmetric() {
    vec4 p0 = vec4(1.0, 2.0, 3.0, 4.0);
    vec4 p1 = vec4(5.0, 6.0, 7.0, 8.0);
    float d1 = distance(p0, p1);
    float d2 = distance(p1, p0);
    // Distance should be symmetric
    if (d1 == d2) {
        return 1.0;
    }
    return 0.0;
    // Should be 1.0 (symmetric)
}

// run: test_vec4_distance_symmetric() ~= 1.0

