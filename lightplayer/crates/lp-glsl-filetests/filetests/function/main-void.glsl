// test run
// target riscv32.fixed32

// ============================================================================
// Main Function Return Type: main() must return void
// ============================================================================

// Note: main() function must return void
// These tests verify void return type requirements

void test_main_void_return() {
    // Helper function that returns void
    void process() {
        // Does nothing, returns void
    }

    process();
}

// run: test_main_void_return() == 0.0

float test_main_void_cannot_return_value() {
    // main() cannot return a value - this would be a compile error
    // We test this concept with regular functions

    void func_returns_void() {
        // This function returns void
        float local_var = 42.0;
        // No return statement needed for void
    }

    func_returns_void();
    return 0.0; // Test function returns float
}

// run: test_main_void_cannot_return_value() ~= 0.0

void test_main_void_explicit_return() {
    // Void functions can have explicit return statements
    void explicit_void_return() {
        float x = 1.0;
        if (x > 0.0) {
            return; // Explicit return for void
        }
        x = x + 1.0;
    }

    explicit_void_return();
}

// run: test_main_void_explicit_return() == 0.0

void test_main_void_multiple_returns() {
    // Void functions can have multiple return points
    void conditional_returns() {
        float value = 5.0;

        if (value < 0.0) {
            return; // Early exit 1
        }

        if (value > 10.0) {
            return; // Early exit 2
        }

        // Continue processing
        value = value * 2.0;
    }

    conditional_returns();
}

// run: test_main_void_multiple_returns() == 0.0

void test_main_void_calls_other_void() {
    // Void functions can call other void functions
    void func_a() {
        // Function A
    }

    void func_b() {
        func_a(); // Call void function
    }

    void func_c() {
        func_b(); // Call another void function
    }

    func_c();
}

// run: test_main_void_calls_other_void() == 0.0

float test_main_void_in_expressions() {
    // Void functions cannot be used in expressions
    // This would be a compile error: float x = void_func();
    // We demonstrate this concept

    void void_func() {
        // Returns nothing
    }

    void_func();
    // Cannot do: return void_func(); - would be error
    return 1.0; // Return a valid value
}

// run: test_main_void_in_expressions() ~= 1.0

void test_main_void_with_statements() {
    // Void functions can contain statements
    void complex_void_func() {
        float a = 1.0;
        float b = 2.0;
        vec2 v = vec2(a, b);
        bool flag = true;

        if (flag) {
            a = a + b;
        }

        for (int i = 0; i < 3; i++) {
            a = a + 1.0;
        }
    }

    complex_void_func();
}

// run: test_main_void_with_statements() == 0.0

void test_main_void_empty() {
    // Void functions can be completely empty
    void empty_func() {
        // Empty body
    }

    empty_func();
}

// run: test_main_void_empty() == 0.0

void test_main_void_parameters() {
    // Void functions can take parameters
    void consume_values(float x, int y, vec2 v) {
        // Consume parameters but return nothing
    }

    consume_values(3.14, 42, vec2(1.0, 2.0));
}

// run: test_main_void_parameters() == 0.0
