// test run
// target riscv32.fixed32

// ============================================================================
// Main Function Parameters: main() must take no parameters
// ============================================================================

// Note: main() function must not take any parameters
// These tests verify parameter requirements

float test_main_no_params() {
    // Functions can take no parameters
    float get_constant() {
        return 3.14159;
    }

    return get_constant();
}

// run: test_main_no_params() ~= 3.14159

float test_main_no_params_vs_with_params() {
    // Contrast: functions can take parameters, but main() cannot
    float add_two(float a, float b) {
        return a + b;
    }

    float add_none() {
        return 42.0;
    }

    return add_two(1.0, 2.0) + add_none();
}

// run: test_main_no_params_vs_with_params() ~= 45.0

void test_main_no_params_void() {
    // Void functions can also take no parameters
    void do_nothing() {
        // Empty function with no parameters
    }

    do_nothing();
}

// run: test_main_no_params_void() == 0.0

float test_main_no_params_globals() {
    // Functions with no parameters can use globals
    float global_value = 100.0;

    float get_global() {
        return global_value;
    }

    return get_global();
}

// run: test_main_no_params_globals() ~= 100.0

vec2 test_main_no_params_vector() {
    // Functions returning vectors with no parameters
    vec2 get_origin() {
        return vec2(0.0, 0.0);
    }

    return get_origin();
}

// run: test_main_no_params_vector() ~= vec2(0.0, 0.0)

int test_main_no_params_computation() {
    // Functions performing computation with no input parameters
    int compute_sum() {
        int a = 10;
        int b = 20;
        int c = 30;
        return a + b + c;
    }

    return compute_sum();
}

// run: test_main_no_params_computation() == 60

bool test_main_no_params_boolean() {
    // Boolean functions with no parameters
    bool always_true() {
        return true;
    }

    bool always_false() {
        return false;
    }

    return always_true() && !always_false();
}

// run: test_main_no_params_boolean() == true

float test_main_no_params_calls_with_params() {
    // Functions with no params can call functions with params
    float multiply(float x, float y) {
        return x * y;
    }

    float compute() {
        return multiply(3.0, 4.0) + multiply(2.0, 5.0);
    }

    return compute();
}

// run: test_main_no_params_calls_with_params() ~= 22.0

float test_main_no_params_locals() {
    // Functions with no params can use local variables
    float local_computation() {
        float a = 1.5;
        float b = 2.5;
        float c = a + b;
        c = c * 2.0;
        return c;
    }

    return local_computation();
}

// run: test_main_no_params_locals() ~= 8.0

mat2 test_main_no_params_matrix() {
    // Matrix functions with no parameters
    mat2 get_identity() {
        return mat2(1.0);
    }

    return get_identity();
}

// run: test_main_no_params_matrix() ~= mat2(1.0, 0.0, 0.0, 1.0)
