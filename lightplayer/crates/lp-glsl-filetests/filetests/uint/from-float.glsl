// test run
// target riscv32.fixed32

// ============================================================================
// Constructor: uint(float) - truncates fractional part toward zero (undefined for negative)
// ============================================================================

uint test_uint_from_float_zero() {
    float f = 0.0;
    return uint(f);
}

// run: test_uint_from_float_zero() == 0u

uint test_uint_from_float_one() {
    float f = 1.0;
    return uint(f);
}

// run: test_uint_from_float_one() == 1u

uint test_uint_from_float_positive_truncate() {
    float f = 1.5;
    return uint(f);
}

// run: test_uint_from_float_positive_truncate() == 1u

uint test_uint_from_float_positive_high() {
    float f = 3.9;
    return uint(f);
}

// run: test_uint_from_float_positive_high() == 3u

uint test_uint_from_float_literal_zero() {
    return uint(0.0);
}

// run: test_uint_from_float_literal_zero() == 0u

uint test_uint_from_float_literal_positive() {
    return uint(42.0);
}

// run: test_uint_from_float_literal_positive() == 42u

uint test_uint_from_float_expression() {
    float a = 5.0;
    float b = 2.5;
    return uint(a - b);
}

// run: test_uint_from_float_expression() == 2u

uint test_uint_from_float_expression_zero() {
    float a = 3.0;
    float b = 3.0;
    return uint(a - b);
}

// run: test_uint_from_float_expression_zero() == 0u

uint test_uint_from_float_large() {
    float f = 4294967295.0;  // UINT_MAX
    return uint(f);
}

// run: test_uint_from_float_large() == 4294967295u

