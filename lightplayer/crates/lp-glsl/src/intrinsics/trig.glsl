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
    
    // CORDIC iterations - unrolled to avoid loop/array usage
    // Each iteration uses atan(2^-i)
    float d;
    float x_new, y_new;
    
    // Iteration 0: atan(1) = 0.7853981633974483
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y;
    y_new = y + d * x;
    z = z - d * 0.7853981633974483;
    x = x_new;
    y = y_new;
    
    // Iteration 1: atan(0.5) = 0.4636476090008061
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.5;
    y_new = y + d * x * 0.5;
    z = z - d * 0.4636476090008061;
    x = x_new;
    y = y_new;
    
    // Iteration 2: atan(0.25) = 0.24497866312686414
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.25;
    y_new = y + d * x * 0.25;
    z = z - d * 0.24497866312686414;
    x = x_new;
    y = y_new;
    
    // Iteration 3: atan(0.125) = 0.12435499454676144
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.125;
    y_new = y + d * x * 0.125;
    z = z - d * 0.12435499454676144;
    x = x_new;
    y = y_new;
    
    // Iteration 4: atan(0.0625) = 0.06241880999595735
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.0625;
    y_new = y + d * x * 0.0625;
    z = z - d * 0.06241880999595735;
    x = x_new;
    y = y_new;
    
    // Iteration 5: atan(0.03125) = 0.031239833430268277
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.03125;
    y_new = y + d * x * 0.03125;
    z = z - d * 0.031239833430268277;
    x = x_new;
    y = y_new;
    
    // Iteration 6: atan(0.015625) = 0.015623728620476831
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.015625;
    y_new = y + d * x * 0.015625;
    z = z - d * 0.015623728620476831;
    x = x_new;
    y = y_new;
    
    // Iteration 7: atan(0.0078125) = 0.007812341060101111
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.0078125;
    y_new = y + d * x * 0.0078125;
    z = z - d * 0.007812341060101111;
    x = x_new;
    y = y_new;
    
    // Iteration 8: atan(0.00390625) = 0.0039062301319669718
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.00390625;
    y_new = y + d * x * 0.00390625;
    z = z - d * 0.0039062301319669718;
    x = x_new;
    y = y_new;
    
    // Iteration 9: atan(0.001953125) = 0.0019531225164788188
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.001953125;
    y_new = y + d * x * 0.001953125;
    z = z - d * 0.0019531225164788188;
    x = x_new;
    y = y_new;
    
    // Iteration 10: atan(0.0009765625) = 0.0009765621895593195
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.0009765625;
    y_new = y + d * x * 0.0009765625;
    z = z - d * 0.0009765621895593195;
    x = x_new;
    y = y_new;
    
    // Iteration 11: atan(0.00048828125) = 0.0004882812111948983
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.00048828125;
    y_new = y + d * x * 0.00048828125;
    z = z - d * 0.0004882812111948983;
    x = x_new;
    y = y_new;
    
    // Iteration 12: atan(0.000244140625) = 0.00024414062014936177
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.000244140625;
    y_new = y + d * x * 0.000244140625;
    z = z - d * 0.00024414062014936177;
    x = x_new;
    y = y_new;
    
    // Iteration 13: atan(0.0001220703125) = 0.00012207031189367021
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 0.0001220703125;
    y_new = y + d * x * 0.0001220703125;
    z = z - d * 0.00012207031189367021;
    x = x_new;
    y = y_new;
    
    // Iteration 14: atan(6.103515625e-05) = 6.103515617420877e-05
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 6.103515625e-05;
    y_new = y + d * x * 6.103515625e-05;
    z = z - d * 6.103515617420877e-05;
    x = x_new;
    y = y_new;
    
    // Iteration 15: atan(3.0517578125e-05) = 3.0517578115526096e-05
    if (z >= 0.0) {
        d = 1.0;
    } else {
        d = -1.0;
    }
    x_new = x - d * y * 3.0517578125e-05;
    y_new = y + d * x * 3.0517578125e-05;
    z = z - d * 3.0517578115526096e-05;
    x = x_new;
    y = y_new;
    
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
