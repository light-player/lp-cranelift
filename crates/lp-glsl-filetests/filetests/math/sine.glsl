// test compile
// test run

float main() {
    // Test key sine values
    // sin(0) = 0
    float s0 = sin(0.0);
    
    // sin(π/2) = 1
    float pi_2 = 1.570796327;
    float s1 = sin(pi_2);
    
    // sin(π) = 0
    float pi = 3.141592654;
    float s2 = sin(pi);
    
    // sin(3π/2) = -1
    float pi_3_2 = 4.712388981;
    float s3 = sin(pi_3_2);
    
    // Sum should be 0.0 (0 + 1 + 0 + (-1))
    float result = s0 + s1 + s2 + s3;
    return result;
}

// Expected CLIF should show:
// - Function __lp_sin is present (not external call %sinf)
// - Helper functions __lp_reduce_angle, __lp_cordic_rotation may be present
// - No external calls to "sinf"

// run: ~= 0.0
