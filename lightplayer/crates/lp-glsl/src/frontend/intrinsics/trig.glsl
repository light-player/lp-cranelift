// Intrinsic trigonometric function implementations using CORDIC algorithm.
// All functions use float types - fixed-point conversion happens automatically.

// Helper: Reduce angle to [0, π/2] range and return quadrant
vec2 __lp_reduce_angle(float angle) {
    float pi = 3.14159265358979323846;
    float pi_2 = 1.57079632679489661923;
    float pi_3_2 = 4.71238898038468985769;
    float two_pi = 6.28318530717958647692;
    
    // Normalize to [0, 2π) - use local variable since parameters are read-only
    float normalized_angle = mod(angle, two_pi);
    if (normalized_angle < 0.0) {
        normalized_angle = normalized_angle + two_pi;
    }
    
    float reduced_angle;
    float quadrant;
    
    if (normalized_angle <= pi_2) {
        // Quadrant 0: [0, π/2]
        reduced_angle = normalized_angle;
        quadrant = 0.0;
    } else if (normalized_angle <= pi) {
        // Quadrant 1: (π/2, π]
        reduced_angle = pi - normalized_angle;
        quadrant = 1.0;
    } else if (normalized_angle <= pi_3_2) {
        // Quadrant 2: (π, 3π/2]
        reduced_angle = normalized_angle - pi;
        quadrant = 2.0;
    } else {
        // Quadrant 3: (3π/2, 2π)
        reduced_angle = two_pi - normalized_angle;
        quadrant = 3.0;
    }
    
    return vec2(reduced_angle, quadrant);
}

// Helper: CORDIC rotation algorithm
// Returns vec2(sin, cos) for the given angle in [0, π/2]
vec2 __lp_cordic_rotation(float angle) {
    // CORDIC gain factor (K = 0.607252935...)
    float gain = 0.6072529350088812561694;
    
    // Initial values
    float x = gain;
    float y = 0.0;
    float z = angle;
    
    // Precomputed atan values for each iteration: atan(2^-i)
    // These are computed once and reused
    float atan_vals[16];
    atan_vals[0] = 0.7853981633974483;      // atan(1)
    atan_vals[1] = 0.4636476090008061;      // atan(0.5)
    atan_vals[2] = 0.24497866312686414;     // atan(0.25)
    atan_vals[3] = 0.12435499454676144;     // atan(0.125)
    atan_vals[4] = 0.06241880999595735;     // atan(0.0625)
    atan_vals[5] = 0.031239833430268277;    // atan(0.03125)
    atan_vals[6] = 0.015623728620476831;    // atan(0.015625)
    atan_vals[7] = 0.007812341060101111;    // atan(0.0078125)
    atan_vals[8] = 0.0039062301319669718;   // atan(0.00390625)
    atan_vals[9] = 0.0019531225164788188;   // atan(0.001953125)
    atan_vals[10] = 0.0009765621895593195;  // atan(0.0009765625)
    atan_vals[11] = 0.0004882812111948983;  // atan(0.00048828125)
    atan_vals[12] = 0.00024414062014936177; // atan(0.000244140625)
    atan_vals[13] = 0.00012207031189367021; // atan(0.0001220703125)
    atan_vals[14] = 6.103515617420877e-05;  // atan(6.103515625e-05)
    atan_vals[15] = 3.0517578115526096e-05; // atan(3.0517578125e-05)
    
    // CORDIC iterations using a loop
    float multiplier = 1.0;
    for (int i = 0; i < 16; i++) {
        float d = (z >= 0.0) ? 1.0 : -1.0;
        float x_new = x - d * y * multiplier;
        float y_new = y + d * x * multiplier;
        z = z - d * atan_vals[i];
        x = x_new;
        y = y_new;
        
        // Update multiplier for next iteration: 2^-i
        multiplier = multiplier * 0.5;
    }
    
    return vec2(y, x); // sin = y, cos = x
}

// Main sine function
float __lp_sin(float angle) {
    vec2 reduced = __lp_reduce_angle(angle);
    float reduced_angle = reduced.x;
    float quadrant = reduced.y;
    
    vec2 result = __lp_cordic_rotation(reduced_angle);
    float sin_val = result.x;
    float cos_val = result.y;
    
    // Apply quadrant transformations
    if (quadrant == 0.0) {
        return sin_val;
    } else if (quadrant == 1.0) {
        return sin_val; // sin(π - x) = sin(x)
    } else if (quadrant == 2.0) {
        return -sin_val; // sin(π + x) = -sin(x)
    } else {
        return -sin_val; // sin(2π - x) = -sin(x)
    }
}

// Cosine function (can use sine with phase shift)
float __lp_cos(float angle) {
    float pi_2 = 1.57079632679489661923;
    return __lp_sin(angle + pi_2);
}

// Tangent function
float __lp_tan(float angle) {
    vec2 reduced = __lp_reduce_angle(angle);
    float reduced_angle = reduced.x;
    float quadrant = reduced.y;
    
    vec2 result = __lp_cordic_rotation(reduced_angle);
    float sin_val = result.x;
    float cos_val = result.y;
    
    // Avoid division by zero
    if (abs(cos_val) < 1e-6) {
        // Return large value (infinity approximation)
        if (cos_val >= 0.0) {
            return 1e6;
        } else {
            return -1e6;
        }
    }
    
    float tan_val = sin_val / cos_val;
    
    // Apply quadrant transformations
    if (quadrant == 0.0) {
        return tan_val;
    } else if (quadrant == 1.0) {
        return -tan_val; // tan(π - x) = -tan(x)
    } else if (quadrant == 2.0) {
        return tan_val; // tan(π + x) = tan(x)
    } else {
        return -tan_val; // tan(2π - x) = -tan(x)
    }
}

// Placeholder implementations for other functions (to be implemented later)
float __lp_asin(float x) {
    // TODO: Implement arc sine
    return 0.0;
}

float __lp_acos(float x) {
    // TODO: Implement arc cosine
    return 0.0;
}

float __lp_atan(float x) {
    // TODO: Implement arc tangent
    return 0.0;
}

float __lp_sinh(float x) {
    // TODO: Implement hyperbolic sine
    return 0.0;
}

float __lp_cosh(float x) {
    // TODO: Implement hyperbolic cosine
    return 0.0;
}

float __lp_tanh(float x) {
    // TODO: Implement hyperbolic tangent
    return 0.0;
}

float __lp_asinh(float x) {
    // TODO: Implement inverse hyperbolic sine
    return 0.0;
}

float __lp_acosh(float x) {
    // TODO: Implement inverse hyperbolic cosine
    return 0.0;
}

float __lp_atanh(float x) {
    // TODO: Implement inverse hyperbolic tangent
    return 0.0;
}

// Dummy main() function required by GLSL parser
// This function is never called - it's only here to satisfy the parser requirement
void main() {
    // Empty - intrinsics are compiled as separate functions
}
