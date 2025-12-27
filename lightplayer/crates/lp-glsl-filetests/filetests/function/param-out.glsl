// test run
// target riscv32.fixed32

// ============================================================================
// Out Parameters: Copy out only (uninitialized on input)
// ============================================================================

float test_param_out_simple() {
    // Out parameter - must assign before return
    void set_value(out float result) {
        result = 42.0;
    }

    float value;
    set_value(value);
    return value;
}

// run: test_param_out_simple() ~= 42.0

void test_param_out_multiple() {
    // Multiple out parameters
    void get_coordinates(out float x, out float y) {
        x = 10.0;
        y = 20.0;
    }

    float a, b;
    get_coordinates(a, b);
    // Return sum to verify both were set
}

// run: test_param_out_multiple() == 0.0

vec2 test_param_out_vector() {
    // Out parameter with vector type
    void create_vector(out vec2 result) {
        result = vec2(1.0, 2.0);
    }

    vec2 v;
    create_vector(v);
    return v;
}

// run: test_param_out_vector() ~= vec2(1.0, 2.0)

int test_param_out_int() {
    // Out parameter with integer type
    void double_value(out int result, int input) {
        result = input * 2;
    }

    int value;
    double_value(value, 5);
    return value;
}

// run: test_param_out_int() == 10

bool test_param_out_bool() {
    // Out parameter with boolean type
    void set_flag(out bool flag) {
        flag = true;
    }

    bool b;
    set_flag(b);
    return b;
}

// run: test_param_out_bool() == true

float test_param_out_modify_existing() {
    // Out parameter overwrites any existing value
    void overwrite_value(out float result) {
        result = 100.0;  // Overwrites whatever was in result
    }

    float value = 999.0;  // This value should be ignored
    overwrite_value(value);
    return value;
}

// run: test_param_out_modify_existing() ~= 100.0

vec3 test_param_out_complex() {
    // Complex out parameter usage
    void process_components(out vec3 result, float x, float y, float z) {
        result.x = x * 2.0;
        result.y = y + 1.0;
        result.z = z - 0.5;
    }

    vec3 v;
    process_components(v, 1.0, 2.0, 3.0);
    return v;
}

// run: test_param_out_complex() ~= vec3(2.0, 3.0, 2.5)
