// test run
// target riscv32.fixed32

// ============================================================================
// Simple Function Definitions: Basic function implementations
// ============================================================================

float test_define_simple_float() {
    // Simple function definition
    float helper(float x) {
        return x * 2.0;
    }

    return helper(5.0);
}

// run: test_define_simple_float() ~= 10.0

void test_define_simple_void() {
    // Void function definition
    void helper() {
        // Empty function body
    }

    helper();
}

// run: test_define_simple_void() == 0.0

int test_define_simple_int() {
    // Integer function definition
    int add_ints(int a, int b) {
        return a + b;
    }

    return add_ints(3, 7);
}

// run: test_define_simple_int() == 10

bool test_define_simple_bool() {
    // Boolean function definition
    bool and_bool(bool a, bool b) {
        return a && b;
    }

    return and_bool(true, false);
}

// run: test_define_simple_bool() == false

float test_define_simple_no_params() {
    // Function with no parameters
    float get_pi() {
        return 3.14159;
    }

    return get_pi();
}

// run: test_define_simple_no_params() ~= 3.14159

float test_define_simple_multiple_stmts() {
    // Function with multiple statements
    float complex_calc(float x) {
        float temp = x * 2.0;
        temp = temp + 1.0;
        return temp / 2.0;
    }

    return complex_calc(4.0);
}

// run: test_define_simple_multiple_stmts() ~= 4.5

float test_define_simple_nested() {
    // Nested function definitions
    float outer_func(float x) {
        float inner_func(float y) {
            return y + 1.0;
        }

        return inner_func(x) * 2.0;
    }

    return outer_func(3.0);
}

// run: test_define_simple_nested() ~= 8.0
