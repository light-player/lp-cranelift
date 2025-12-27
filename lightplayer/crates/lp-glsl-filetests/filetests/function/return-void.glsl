// test run
// target riscv32.fixed32

// ============================================================================
// Void Return Type: Functions that return nothing
// ============================================================================

void test_return_void_empty() {
    // Void function with empty body
    void do_nothing() {
    }

    do_nothing();
}

// run: test_return_void_empty() == 0.0

void test_return_void_explicit() {
    // Void function with explicit return
    void explicit_return() {
        return;
    }

    explicit_return();
}

// run: test_return_void_explicit() == 0.0

void test_return_void_with_statements() {
    // Void function with statements but no return value
    void process_data() {
        float x = 1.0;
        x = x + 1.0;
        // No return statement needed for void functions
    }

    process_data();
}

// run: test_return_void_with_statements() == 0.0

void test_return_void_multiple_calls() {
    // Multiple calls to void functions
    void func_a() {
        // Function A
    }

    void func_b() {
        // Function B
    }

    func_a();
    func_b();
    func_a();
}

// run: test_return_void_multiple_calls() == 0.0

void test_return_void_in_sequence() {
    // Void functions called in sequence
    void step1() {
        // Step 1
    }

    void step2() {
        // Step 2
    }

    void step3() {
        // Step 3
    }

    step1();
    step2();
    step3();
}

// run: test_return_void_in_sequence() == 0.0

void test_return_void_with_params() {
    // Void function with parameters
    void consume_value(float value) {
        // Consume the value (do nothing with it)
    }

    consume_value(42.0);
    consume_value(3.14);
}

// run: test_return_void_with_params() == 0.0

void test_return_void_nested_calls() {
    // Nested calls to void functions
    void inner() {
        // Inner void function
    }

    void outer() {
        inner();
        inner();
    }

    outer();
}

// run: test_return_void_nested_calls() == 0.0
