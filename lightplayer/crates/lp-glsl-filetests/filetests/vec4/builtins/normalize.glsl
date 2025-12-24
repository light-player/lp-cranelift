// test run
// target riscv32.fixed32

// ============================================================================
// Normalize: normalize(vec4) - returns unit vector (length = 1.0)
// normalize(v) = v / length(v)
// ============================================================================

float test_vec4_normalize_length() {
    vec4 v = vec4(3.0, 4.0, 0.0, 0.0);
    vec4 result = normalize(v);
    // Length of normalized vector should be 1.0
    // Original length = sqrt(9 + 16) = 5.0
    // Normalized = (3/5, 4/5, 0, 0) = (0.6, 0.8, 0, 0)
    return length(result);
    // Should be approximately 1.0
}

// run: test_vec4_normalize_length() ~= 1.0

float test_vec4_normalize_components() {
    vec4 v = vec4(3.0, 4.0, 0.0, 0.0);
    vec4 result = normalize(v);
    // Verify normalized components
    // Original length = 5.0, so normalized = (3/5, 4/5, 0, 0) = (0.6, 0.8, 0, 0)
    return result.x + result.y + result.z + result.w;
    // Should be approximately 0.6 + 0.8 + 0.0 + 0.0 = 1.4
}

// run: test_vec4_normalize_components() ~= 1.4

float test_vec4_normalize_unit() {
    vec4 v = vec4(1.0, 0.0, 0.0, 0.0);
    vec4 result = normalize(v);
    // Normalize unit vector (should remain unchanged)
    return result.x + result.y + result.z + result.w;
    // Should be 1.0 + 0.0 + 0.0 + 0.0 = 1.0
}

// run: test_vec4_normalize_unit() ~= 1.0

float test_vec4_normalize_all_components() {
    vec4 v = vec4(2.0, 2.0, 2.0, 2.0);
    vec4 result = normalize(v);
    // Normalize vector with all equal components
    // Length = sqrt(4+4+4+4) = sqrt(16) = 4.0
    // Normalized = (2/4, 2/4, 2/4, 2/4) = (0.5, 0.5, 0.5, 0.5)
    return result.x + result.y + result.z + result.w;
    // Should be 0.5 + 0.5 + 0.5 + 0.5 = 2.0
}

// run: test_vec4_normalize_all_components() ~= 2.0

float test_vec4_normalize_verify_direction() {
    vec4 v = vec4(6.0, 8.0, 0.0, 0.0);
    vec4 result = normalize(v);
    // Verify direction preserved (proportional)
    // Original: (6, 8, 0, 0), length = 10.0
    // Normalized: (0.6, 0.8, 0, 0)
    // Ratio should be preserved: 6/8 = 0.6/0.8 = 0.75
    float ratio_original = v.x / v.y;
    float ratio_normalized = result.x / result.y;
    if (ratio_original == ratio_normalized) {
        return 1.0;
    }
    return 0.0;
    // Should be 1.0 (ratio preserved)
}

// run: test_vec4_normalize_verify_direction() ~= 1.0

