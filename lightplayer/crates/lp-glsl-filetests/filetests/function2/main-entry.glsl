// test run
// target riscv32.fixed32

// ============================================================================
// Main Function Requirements: main() is the entry point
// ============================================================================

// Note: In GLSL, main() is the entry point for shader execution
// In ESSL, main() may be required at compile time
// These tests verify main() function requirements

float helper() {
    return 42.0;
}

float test_main_entry_exists() {
    // Helper function - main() should exist as entry point
    return helper();
}

// run: test_main_entry_exists() ~= 42.0

// The main() function would be defined at the global scope
// For testing purposes, we simulate main() requirements

float compute() {
    return 3.14 * 2.0;
}

float test_main_can_call_functions() {
    // main() can call other functions
    float result = compute();
    return result;
}

// run: test_main_can_call_functions() ~= 6.28

float global_value = 100.0;

float get_global() {
    return global_value;
}

float test_main_access_globals() {
    // main() can access global variables
    return get_global();
}

// run: test_main_access_globals() ~= 100.0

float func_a() {
    return 10.0;
}

float func_b() {
    return 20.0;
}

float combine() {
    return func_a() + func_b();
}

float test_main_multiple_functions() {
    // main() can orchestrate multiple function calls
    return combine();
}

// run: test_main_multiple_functions() ~= 30.0

vec2 create_vector() {
    return vec2(1.0, 2.0);
}

vec2 scale_vector(vec2 v) {
    return v * 2.0;
}

vec2 test_main_vector_operations() {
    // main() can perform vector operations
    vec2 v = create_vector();
    vec2 scaled = scale_vector(v);
    return scaled;
}

// run: test_main_vector_operations() ~= vec2(2.0, 4.0)

bool is_positive(float x) {
    return x > 0.0;
}

float process(float x) {
    if (is_positive(x)) {
        return x * 2.0;
    } else {
        return 0.0;
    }
}

float test_main_complex_logic() {
    // main() can contain complex logic
    return process(5.0) + process(-3.0);
}

// run: test_main_complex_logic() ~= 10.0

int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    return a * b;
}

int test_main_integer_ops() {
    // main() can perform integer operations
    return add(5, 3) + multiply(2, 4);
}

// run: test_main_integer_ops() == 16

bool check_range(float x, float min_val, float max_val) {
    return x >= min_val && x <= max_val;
}

bool test_main_boolean_logic() {
    // main() can perform boolean operations
    return check_range(5.0, 0.0, 10.0) && !check_range(15.0, 0.0, 10.0);
}

// run: test_main_boolean_logic() == true
