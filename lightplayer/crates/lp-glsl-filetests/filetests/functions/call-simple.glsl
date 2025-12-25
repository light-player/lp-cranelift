// test run
// target riscv32.fixed32

// ============================================================================
// Simple Function Calls: Basic function invocation
// ============================================================================

float test_call_simple_single_arg() {
    // Call function with single argument
    float square(float x) {
        return x * x;
    }

    return square(4.0);
}

// run: test_call_simple_single_arg() ~= 16.0

float test_call_simple_multiple_args() {
    // Call function with multiple arguments
    float add(float a, float b) {
        return a + b;
    }

    return add(3.0, 7.0);
}

// run: test_call_simple_multiple_args() ~= 10.0

void test_call_simple_void() {
    // Call void function
    void print_value(float x) {
        // In a real shader this would output, but we just call it
    }

    print_value(42.0);
}

// run: test_call_simple_void() == 0.0

float test_call_simple_no_args() {
    // Call function with no arguments
    float get_constant() {
        return 3.14;
    }

    return get_constant();
}

// run: test_call_simple_no_args() ~= 3.14

float test_call_simple_in_expression() {
    // Function call in expression
    float double(float x) {
        return x * 2.0;
    }

    return double(5.0) + double(3.0);
}

// run: test_call_simple_in_expression() ~= 16.0

float test_call_simple_nested_calls() {
    // Nested function calls
    float add_one(float x) {
        return x + 1.0;
    }

    float multiply_by_two(float x) {
        return x * 2.0;
    }

    return multiply_by_two(add_one(3.0));
}

// run: test_call_simple_nested_calls() ~= 8.0

float test_call_simple_in_assignment() {
    // Function call in assignment
    float get_value() {
        return 42.0;
    }

    float result = get_value();
    return result;
}

// run: test_call_simple_in_assignment() ~= 42.0

bool test_call_simple_in_condition() {
    // Function call in conditional expression
    bool is_positive(float x) {
        return x > 0.0;
    }

    return is_positive(5.0) && is_positive(-3.0);
}

// run: test_call_simple_in_condition() == false
