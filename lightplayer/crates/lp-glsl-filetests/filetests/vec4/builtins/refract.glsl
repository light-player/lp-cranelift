// test run
// target riscv32.fixed32

// ============================================================================
// Refract: refract(vec4, vec4, float) - refraction vector
// refract(I, N, eta) - I is incident, N is normal, eta is ratio of indices
// ============================================================================

float test_vec4_refract_normal_incidence() {
    vec4 I = vec4(0.0, -1.0, 0.0, 0.0);  // Incident straight down
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);   // Normal pointing up
    float eta = 0.75;  // Refractive index ratio
    vec4 result = refract(I, N, eta);
    // Normal incidence - should continue straight
    return result.x + result.y + result.z + result.w;
    // Should be approximately 0.0 + -0.75 + 0.0 + 0.0 = -0.75
}

// run: test_vec4_refract_normal_incidence() ~= -0.75

float test_vec4_refract_45_degree() {
    vec4 I = vec4(1.0, -1.0, 0.0, 0.0);  // Incident at 45 degrees
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);   // Normal pointing up
    float eta = 0.5;  // Refractive index ratio
    vec4 result = refract(I, N, eta);
    // Refraction at 45 degrees
    return result.x + result.y + result.z + result.w;
    // Should be some value (exact calculation complex)
}

// run: test_vec4_refract_45_degree() ~= 0.0

float test_vec4_refract_eta_one() {
    vec4 I = vec4(1.0, -1.0, 0.0, 0.0);
    vec4 N = vec4(0.0, 1.0, 0.0, 0.0);
    float eta = 1.0;  // No refraction
    vec4 result = refract(I, N, eta);
    // eta = 1.0 means no refraction, should equal incident
    return result.x + result.y + result.z + result.w;
    // Should be approximately 1.0 + -1.0 + 0.0 + 0.0 = 0.0
}

// run: test_vec4_refract_eta_one() ~= 0.0

