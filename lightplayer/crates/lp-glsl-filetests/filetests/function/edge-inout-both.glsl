// test run
// target riscv32.fixed32

// ============================================================================
// Inout Parameters Both Ways: Copied in and copied out
// ============================================================================

float test_edge_inout_both_directions() {
    // Inout parameter is copied in at start, copied out at end
    void modify_inout(inout float value) {
        value = value + 10.0; // Modifies caller's variable
    }

    float x = 5.0;
    modify_inout(x);
    return x; // Should be 15.0
}

// run: test_edge_inout_both_directions() ~= 15.0

void test_edge_inout_copy_in() {
    // Value is copied in at function entry
    void read_inout(inout float value) {
        float local = value; // local gets the input value
        value = local + 1.0; // Modify for output
    }

    float x = 100.0;
    read_inout(x);
    // x should be 101.0
}

// run: test_edge_inout_copy_in() == 0.0

void test_edge_inout_copy_out() {
    // Value is copied out at function exit
    void write_inout(inout float value) {
        value = 42.0; // This gets copied back to caller
    }

    float x = 999.0; // This will be overwritten
    write_inout(x);
    // x should be 42.0
}

// run: test_edge_inout_copy_out() == 0.0

vec2 test_edge_inout_vector_both() {
    // Vector inout parameters work both ways
    void transform_vector(inout vec2 v) {
        v = v * 2.0; // Double the input vector
        v = v + vec2(1.0, 1.0); // Add offset
    }

    vec2 vec = vec2(1.0, 2.0);
    transform_vector(vec);
    return vec; // Should be vec2(3.0, 5.0)
}

// run: test_edge_inout_vector_both() ~= vec2(3.0, 5.0)

int test_edge_inout_int_both() {
    // Integer inout parameters
    void increment_inout(inout int value) {
        value = value + 1; // Increment the input
    }

    int x = 10;
    increment_inout(x);
    return x; // Should be 11
}

// run: test_edge_inout_int_both() == 11

float test_edge_inout_multiple_operations() {
    // Multiple operations on inout parameter
    void complex_inout(inout float value) {
        value = value * 2.0;  // Double input
        value = value + 5.0;  // Add 5
        value = value / 3.0;  // Divide by 3
    }

    float x = 6.0;
    complex_inout(x);
    return x; // ((6*2)+5)/3 = 17/3 = 5.666...
}

// run: test_edge_inout_both_multiple_operations() ~= 5.666

void test_edge_inout_unchanged_if_not_modified() {
    // If inout parameter is not modified, original value preserved
    void no_modify(inout float value) {
        float local = value; // Read input
        // Don't modify value
    }

    float x = 123.0;
    no_modify(x);
    // x should still be 123.0
}

// run: test_edge_inout_unchanged_if_not_modified() == 0.0

bool test_edge_inout_bool_both() {
    // Boolean inout parameters
    void toggle_inout(inout bool flag) {
        flag = !flag; // Toggle the input boolean
    }

    bool b = false;
    toggle_inout(b);
    return b; // Should be true
}

// run: test_edge_inout_bool_both() == true

float test_edge_inout_expression() {
    // Inout can be used in expressions
    void use_in_expression(inout float value) {
        float doubled = value * 2.0;
        value = doubled + value; // value = 2*value + value = 3*value
    }

    float x = 4.0;
    use_in_expression(x);
    return x; // Should be 12.0
}

// run: test_edge_inout_expression() ~= 12.0
