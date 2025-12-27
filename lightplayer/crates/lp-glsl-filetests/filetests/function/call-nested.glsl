// test run
// target riscv32.fixed32

// ============================================================================
// Nested Function Calls: Functions calling other functions
// ============================================================================

float test_call_nested_simple() {
    // Simple nested calls
    float double(float x) {
        return x * 2.0;
    }

    float add_one(float x) {
        return x + 1.0;
    }

    return double(add_one(3.0)); // double(add_one(3)) = double(4) = 8
}

// run: test_call_nested_simple() ~= 8.0

float test_call_nested_deep() {
    // Deep nesting
    float func_a(float x) {
        return x + 1.0;
    }

    float func_b(float x) {
        return func_a(x) * 2.0;
    }

    float func_c(float x) {
        return func_b(x) + 3.0;
    }

    return func_c(2.0); // func_b(func_a(2)) + 3 = func_b(3)*2 + 3 = 6 + 3 = 9
}

// run: test_call_nested_deep() ~= 9.0

vec2 test_call_nested_vector() {
    // Nested calls with vectors
    vec2 scale(vec2 v, float s) {
        return v * s;
    }

    vec2 rotate(vec2 v) {
        return vec2(-v.y, v.x);
    }

    vec2 transform(vec2 v) {
        return scale(rotate(v), 2.0);
    }

    vec2 input = vec2(1.0, 0.0);
    return transform(input); // scale(rotate(vec2(1,0)), 2) = scale(vec2(0,1), 2) = vec2(0,2)
}

// run: test_call_nested_vector() ~= vec2(0.0, 2.0)

float test_call_nested_mixed_types() {
    // Nested calls with different return types
    float get_length(vec2 v) {
        return length(v);
    }

    int round_to_int(float x) {
        return int(x + 0.5);
    }

    float process_vector(vec2 v) {
        return float(round_to_int(get_length(v)));
    }

    return process_vector(vec2(3.0, 4.0)); // round_to_int(length(vec2(3,4))) = round_to_int(5) = 5
}

// run: test_call_nested_mixed_types() ~= 5.0

float test_call_nested_multiple_paths() {
    // Multiple nested call paths
    float square(float x) {
        return x * x;
    }

    float cube(float x) {
        return x * x * x;
    }

    float complex_calc(float a, float b) {
        return square(a) + cube(b);
    }

    return complex_calc(2.0, 3.0); // square(2) + cube(3) = 4 + 27 = 31
}

// run: test_call_nested_multiple_paths() ~= 31.0

mat2 test_call_nested_matrix() {
    // Nested calls with matrices
    mat2 identity() {
        return mat2(1.0);
    }

    mat2 scale_matrix(float s) {
        return mat2(s, 0.0, 0.0, s);
    }

    mat2 combine_transforms(mat2 a, mat2 b) {
        return a * b;
    }

    return combine_transforms(scale_matrix(2.0), identity());
}

// run: test_call_nested_matrix() ~= mat2(2.0, 0.0, 0.0, 2.0)

float test_call_nested_recursive_pattern() {
    // Non-recursive nested pattern
    float apply_twice(float x, float factor) {
        return x * factor * factor;
    }

    float process(float x) {
        return apply_twice(x, 3.0);
    }

    return process(2.0); // apply_twice(2, 3) = 2 * 3 * 3 = 18
}

// run: test_call_nested_recursive_pattern() ~= 18.0

float test_call_nested_chained() {
    // Chained nested calls
    float increment(float x) {
        return x + 1.0;
    }

    // Chain of increments: ((2+1)+1)+1 = 5
    return increment(increment(increment(2.0)));
}

// run: test_call_nested_chained() ~= 5.0
