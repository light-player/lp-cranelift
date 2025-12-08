//! Integration test that runs all filetests

use std::path::PathBuf;

fn test_file(path: &str) {
    let mut full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    full_path.push("filetests");
    full_path.push(path);

    lp_glsl_filetests::run_filetest(&full_path).unwrap();
}

#[test]
fn test_int_literal() {
    test_file("basic/int_literal.glsl");
}

#[test]
fn test_bool_literal() {
    test_file("basic/bool_literal.glsl");
}

#[test]
fn test_arithmetic() {
    test_file("basic/arithmetic.glsl");
}

#[test]
fn test_comparisons() {
    test_file("basic/comparisons.glsl");
}

#[test]
fn test_unary() {
    test_file("basic/unary.glsl");
}

#[test]
fn test_complex_expr() {
    test_file("basic/complex_expr.glsl");
}

#[test]
fn test_assignment() {
    test_file("basic/assignment.glsl");
}

#[test]
fn test_bool_not() {
    test_file("basic/bool_not.glsl");
}

// Control flow tests
#[test]
fn test_simple_if() {
    test_file("control_flow/simple_if.glsl");
}

#[test]
fn test_if_else() {
    test_file("control_flow/if_else.glsl");
}

#[test]
fn test_while_loop() {
    test_file("control_flow/while_loop.glsl");
}

#[test]
fn test_for_loop() {
    test_file("control_flow/for_loop.glsl");
}

#[test]
fn test_break_stmt() {
    test_file("control_flow/break_stmt.glsl");
}

#[test]
fn test_continue_stmt() {
    test_file("control_flow/continue_stmt.glsl");
}

#[test]
fn test_nested_loops() {
    test_file("control_flow/nested_loops.glsl");
}

#[test]
fn test_early_return() {
    test_file("control_flow/early_return.glsl");
}

// Interpolation function tests
#[test]
fn test_mix_scalar() {
    test_file("builtins/interpolation/mix_scalar.glsl");
}

#[test]
fn test_mix_vec3() {
    test_file("builtins/interpolation/mix_vec3.glsl");
}

#[test]
fn test_mix_vec3_scalar_a() {
    test_file("builtins/interpolation/mix_vec3_scalar_a.glsl");
}

#[test]
fn test_step_scalar() {
    test_file("builtins/interpolation/step_scalar.glsl");
}

#[test]
fn test_step_vec3() {
    test_file("builtins/interpolation/step_vec3.glsl");
}

#[test]
fn test_step_scalar_edge() {
    test_file("builtins/interpolation/step_scalar_edge.glsl");
}

#[test]
fn test_smoothstep_scalar() {
    test_file("builtins/interpolation/smoothstep_scalar.glsl");
}

#[test]
fn test_smoothstep_vec3() {
    test_file("builtins/interpolation/smoothstep_vec3.glsl");
}

#[test]
fn test_smoothstep_scalar_edges() {
    test_file("builtins/interpolation/smoothstep_scalar_edges.glsl");
}

// Common function tests (continued)
#[test]
fn test_fract_scalar() {
    test_file("builtins/common/fract_scalar.glsl");
}

#[test]
fn test_fract_vec2() {
    test_file("builtins/common/fract_vec2.glsl");
}

#[test]
fn test_mod_scalar() {
    test_file("builtins/common/mod_scalar.glsl");
}

#[test]
fn test_mod_vec3() {
    test_file("builtins/common/mod_vec3.glsl");
}

#[test]
fn test_mod_vec3_scalar() {
    test_file("builtins/common/mod_vec3_scalar.glsl");
}

#[test]
fn test_sign_float() {
    test_file("builtins/common/sign_float.glsl");
}

#[test]
fn test_sign_int() {
    test_file("builtins/common/sign_int.glsl");
}

// Fixed-point transformation tests - SSA level
#[test]
fn test_fixed_const_16x16() {
    test_file("fixed_point/const_16x16.glsl");
}

#[test]
fn test_fixed_add_16x16() {
    test_file("fixed_point/add_16x16.glsl");
}

#[test]
fn test_fixed_sub_16x16() {
    test_file("fixed_point/sub_16x16.glsl");
}

#[test]
fn test_fixed_mul_16x16() {
    test_file("fixed_point/mul_16x16.glsl");
}

#[test]
fn test_fixed_div_16x16() {
    test_file("fixed_point/div_16x16.glsl");
}

#[test]
fn test_fixed_cmp_eq_16x16() {
    test_file("fixed_point/cmp_eq_16x16.glsl");
}

#[test]
fn test_fixed_cmp_lt_16x16() {
    test_file("fixed_point/cmp_lt_16x16.glsl");
}

#[test]
fn test_fixed_expr_complex_16x16() {
    test_file("fixed_point/expr_complex_16x16.glsl");
}

#[test]
fn test_fixed_expr_nested_16x16() {
    test_file("fixed_point/expr_nested_16x16.glsl");
}

#[test]
fn test_fixed_negative_values_16x16() {
    test_file("fixed_point/negative_values_16x16.glsl");
}

#[test]
fn test_fixed_small_fractions_16x16() {
    test_file("fixed_point/small_fractions_16x16.glsl");
}

#[test]
fn test_fixed_range_limits_16x16() {
    test_file("fixed_point/range_limits_16x16.glsl");
}

#[test]
fn test_fixed_const_32x32() {
    test_file("fixed_point/const_32x32.glsl");
}

#[test]
fn test_fixed_mul_32x32() {
    test_file("fixed_point/mul_32x32.glsl");
}

#[test]
fn test_fixed_div_32x32() {
    test_file("fixed_point/div_32x32.glsl");
}

// Fixed-point transformation tests - Runtime
#[test]
fn test_fixed_run_add_16x16() {
    test_file("fixed_point/run_add_16x16.glsl");
}

#[test]
fn test_fixed_run_mul_16x16() {
    test_file("fixed_point/run_mul_16x16.glsl");
}

#[test]
fn test_fixed_run_div_16x16() {
    test_file("fixed_point/run_div_16x16.glsl");
}

#[test]
fn test_fixed_run_precision_add_16x16() {
    test_file("fixed_point/run_precision_add_16x16.glsl");
}

#[test]
fn test_fixed_run_precision_mul_chain_16x16() {
    test_file("fixed_point/run_precision_mul_chain_16x16.glsl");
}

#[test]
fn test_fixed_run_add_32x32() {
    test_file("fixed_point/run_add_32x32.glsl");
}

#[test]
fn test_fixed_run_mul_32x32() {
    test_file("fixed_point/run_mul_32x32.glsl");
}

#[test]
fn test_fixed_run_negative_16x16() {
    test_file("fixed_point/run_negative_16x16.glsl");
}

#[test]
fn test_fixed_run_near_zero_16x16() {
    test_file("fixed_point/run_near_zero_16x16.glsl");
}

#[test]
fn test_fixed_run_cmp_eq_16x16() {
    test_file("fixed_point/run_cmp_eq_16x16.glsl");
}

#[test]
fn test_fixed_run_cmp_lt_16x16() {
    test_file("fixed_point/run_cmp_lt_16x16.glsl");
}

// Float tests
#[test]
fn test_float_literal() {
    test_file("float/float_literal.glsl");
}

#[test]
fn test_float_arithmetic() {
    test_file("float/float_arithmetic.glsl");
}

#[test]
fn test_float_multiplication() {
    test_file("float/float_multiplication.glsl");
}

#[test]
fn test_float_comparison() {
    test_file("float/float_comparison.glsl");
}

#[test]
fn test_int_to_float() {
    test_file("float/int_to_float.glsl");
}

#[test]
fn test_float_assignment() {
    test_file("float/float_assignment.glsl");
}

#[test]
fn test_mixed_arithmetic() {
    test_file("float/mixed_arithmetic.glsl");
}

#[test]
fn test_float_in_loop() {
    test_file("float/float_in_loop.glsl");
}

// Vector construction tests
#[test]
fn test_vec3_construct() {
    test_file("vectors/vec3_construct.glsl");
}

#[test]
fn test_vec3_broadcast() {
    test_file("vectors/vec3_broadcast.glsl");
}

#[test]
fn test_ivec2_construct() {
    test_file("vectors/ivec2_construct.glsl");
}

#[test]
fn test_vec3_from_ivec3() {
    test_file("vectors/vec3_from_ivec3.glsl");
}

#[test]
fn test_vec4_concat() {
    test_file("vectors/vec4_concat.glsl");
}

#[test]
fn test_vec3_int_to_float() {
    test_file("vectors/vec3_int_to_float.glsl");
}

// Vector arithmetic tests
#[test]
fn test_vec3_add() {
    test_file("vectors/vec3_add.glsl");
}

#[test]
fn test_vec3_multiply_scalar() {
    test_file("vectors/vec3_multiply_scalar.glsl");
}

#[test]
fn test_scalar_multiply_vec3() {
    test_file("vectors/scalar_multiply_vec3.glsl");
}

#[test]
fn test_ivec2_subtract() {
    test_file("vectors/ivec2_subtract.glsl");
}

#[test]
fn test_vec4_divide_scalar() {
    test_file("vectors/vec4_divide_scalar.glsl");
}

#[test]
fn test_vec3_mixed_ops() {
    test_file("vectors/vec3_mixed_ops.glsl");
}

// Component access tests
#[test]
fn test_component_access_x() {
    test_file("vectors/component_access_x.glsl");
}

#[test]
fn test_component_access_rgba() {
    test_file("vectors/component_access_rgba.glsl");
}

#[test]
fn test_component_assign() {
    test_file("vectors/component_assign.glsl");
}

#[test]
fn test_ivec2_component() {
    test_file("vectors/ivec2_component.glsl");
}

// Geometric function tests
#[test]
fn test_dot_vec3() {
    test_file("builtins/geometric/dot_vec3.glsl");
}

#[test]
fn test_cross_vec3() {
    test_file("builtins/geometric/cross_vec3.glsl");
}

#[test]
fn test_length_vec2() {
    test_file("builtins/geometric/length_vec2.glsl");
}

#[test]
fn test_normalize_vec3() {
    test_file("builtins/geometric/normalize_vec3.glsl");
}

#[test]
fn test_distance_vec3() {
    test_file("builtins/geometric/distance_vec3.glsl");
}

// Common function tests
#[test]
fn test_min_scalars() {
    test_file("builtins/common/min_scalars.glsl");
}

#[test]
fn test_min_vec3_scalar() {
    test_file("builtins/common/min_vec3_scalar.glsl");
}

#[test]
fn test_max_vec2() {
    test_file("builtins/common/max_vec2.glsl");
}

#[test]
fn test_clamp_scalar() {
    test_file("builtins/common/clamp_scalar.glsl");
}

#[test]
fn test_clamp_vec3() {
    test_file("builtins/common/clamp_vec3.glsl");
}

#[test]
fn test_abs_float() {
    test_file("builtins/common/abs_float.glsl");
}

#[test]
fn test_abs_vec2() {
    test_file("builtins/common/abs_vec2.glsl");
}

#[test]
fn test_sqrt_scalar() {
    test_file("builtins/common/sqrt_scalar.glsl");
}

#[test]
fn test_binary_compilation() {
    test_file("builtins/test_binary_compilation.glsl");
}

#[test]
fn test_sqrt_vec3() {
    test_file("builtins/common/sqrt_vec3.glsl");
}

#[test]
fn test_floor_scalar() {
    test_file("builtins/common/floor_scalar.glsl");
}

#[test]
fn test_floor_vec2() {
    test_file("builtins/common/floor_vec2.glsl");
}

#[test]
fn test_ceil_scalar() {
    test_file("builtins/common/ceil_scalar.glsl");
}

#[test]
fn test_ceil_vec3() {
    test_file("builtins/common/ceil_vec3.glsl");
}

#[test]
fn test_pow_error() {
    test_file("builtins/common/pow_error.glsl");
}

// Trigonometric function tests
#[test]
fn test_radians_scalar() {
    test_file("builtins/trigonometric/radians_scalar.glsl");
}

#[test]
fn test_radians_vec2() {
    test_file("builtins/trigonometric/radians_vec2.glsl");
}

#[test]
fn test_degrees_scalar() {
    test_file("builtins/trigonometric/degrees_scalar.glsl");
}

#[test]
fn test_radians_degrees_roundtrip() {
    test_file("builtins/trigonometric/radians_degrees_roundtrip.glsl");
}

#[test]
fn test_sin_scalar() {
    test_file("builtins/trigonometric/sin_scalar.glsl");
}

#[test]
fn test_sin_pi_2() {
    test_file("builtins/trigonometric/sin_pi_2.glsl");
}

#[test]
fn test_sin_vec3() {
    test_file("builtins/trigonometric/sin_vec3.glsl");
}

#[test]
fn test_cos_scalar() {
    test_file("builtins/trigonometric/cos_scalar.glsl");
}

#[test]
fn test_cos_pi() {
    test_file("builtins/trigonometric/cos_pi.glsl");
}

#[test]
fn test_cos_vec2() {
    test_file("builtins/trigonometric/cos_vec2.glsl");
}

#[test]
fn test_tan_scalar() {
    test_file("builtins/trigonometric/tan_scalar.glsl");
}

#[test]
fn test_tan_vec3() {
    test_file("builtins/trigonometric/tan_vec3.glsl");
}

#[test]
fn test_asin_scalar() {
    test_file("builtins/trigonometric/asin_scalar.glsl");
}

#[test]
fn test_asin_boundary() {
    test_file("builtins/trigonometric/asin_boundary.glsl");
}

#[test]
fn test_acos_scalar() {
    test_file("builtins/trigonometric/acos_scalar.glsl");
}

#[test]
fn test_atan_scalar() {
    test_file("builtins/trigonometric/atan_scalar.glsl");
}

#[test]
fn test_atan2_scalar() {
    test_file("builtins/trigonometric/atan2_scalar.glsl");
}

#[test]
fn test_atan2_quadrant() {
    test_file("builtins/trigonometric/atan2_quadrant.glsl");
}

#[test]
fn test_sinh_scalar() {
    test_file("builtins/trigonometric/sinh_scalar.glsl");
}

#[test]
fn test_cosh_scalar() {
    test_file("builtins/trigonometric/cosh_scalar.glsl");
}

#[test]
fn test_tanh_scalar() {
    test_file("builtins/trigonometric/tanh_scalar.glsl");
}

#[test]
fn test_asinh_scalar() {
    test_file("builtins/trigonometric/asinh_scalar.glsl");
}

#[test]
fn test_acosh_scalar() {
    test_file("builtins/trigonometric/acosh_scalar.glsl");
}

#[test]
fn test_atanh_scalar() {
    test_file("builtins/trigonometric/atanh_scalar.glsl");
}

// Matrix function tests
#[test]
fn test_matrixCompMult_mat2() {
    test_file("builtins/matrix/matrixCompMult_mat2.glsl");
}

#[test]
fn test_outerProduct_vec3() {
    test_file("builtins/matrix/outerProduct_vec3.glsl");
}

#[test]
fn test_transpose_mat3() {
    test_file("builtins/matrix/transpose_mat3.glsl");
}

#[test]
fn test_determinant_mat2() {
    test_file("builtins/matrix/determinant_mat2.glsl");
}

#[test]
fn test_determinant_mat3() {
    test_file("builtins/matrix/determinant_mat3.glsl");
}

#[test]
fn test_inverse_mat2() {
    test_file("builtins/matrix/inverse_mat2.glsl");
}

// User function tests
#[test]
fn test_simple_function() {
    test_file("functions/simple_function.glsl");
}

#[test]
fn test_vec3_function() {
    test_file("functions/vec3_function.glsl");
}

#[test]
fn test_multiple_params() {
    test_file("functions/multiple_params.glsl");
}

#[test]
fn test_function_composition() {
    test_file("functions/function_composition.glsl");
}

#[test]
fn test_implicit_conversion_func() {
    test_file("functions/implicit_conversion.glsl");
}

#[test]
fn test_mat2_function() {
    test_file("functions/mat2_function.glsl");
}

#[test]
fn test_mat3_function() {
    test_file("functions/mat3_function.glsl");
}

#[test]
fn test_mat4_function() {
    test_file("functions/mat4_function.glsl");
}

#[test]
fn test_mat3_composition() {
    test_file("functions/mat3_composition.glsl");
}

#[test]
fn test_mat_return_from_builtin() {
    test_file("functions/mat_return_from_builtin.glsl");
}

// Type error tests
#[test]
fn test_bool_plus_int_error() {
    test_file("type_errors/bool_plus_int.glsl");
}

#[test]
fn test_float_to_int_assign_error() {
    test_file("type_errors/float_to_int_assign.glsl");
}

#[test]
fn test_int_condition_error() {
    test_file("type_errors/int_condition.glsl");
}

#[test]
fn test_bool_arithmetic_error() {
    test_file("type_errors/bool_arithmetic.glsl");
}

#[test]
fn test_type_mismatch_assign_error() {
    test_file("type_errors/type_mismatch_assign.glsl");
}

#[test]
fn test_incompatible_comparison_error() {
    test_file("type_errors/incompatible_comparison.glsl");
}

#[test]
fn test_vec_wrong_component_count_error() {
    test_file("type_errors/vec_wrong_component_count.glsl");
}

#[test]
fn test_vec_wrong_vector_size_error() {
    test_file("type_errors/vec_wrong_vector_size.glsl");
}

#[test]
fn test_vec_bool_construct_error() {
    test_file("type_errors/vec_bool_construct.glsl");
}

#[test]
fn test_vec_add_wrong_size_error() {
    test_file("type_errors/vec_add_wrong_size.glsl");
}

#[test]
fn test_vec_component_out_of_range_error() {
    test_file("type_errors/vec_component_out_of_range.glsl");
}

#[test]
fn test_scalar_component_access_error() {
    test_file("type_errors/scalar_component_access.glsl");
}

#[test]
fn test_component_assign_vector_error() {
    test_file("type_errors/component_assign_vector.glsl");
}

#[test]
fn test_dot_size_mismatch_error() {
    test_file("type_errors/dot_size_mismatch.glsl");
}

#[test]
fn test_cross_not_vec3_error() {
    test_file("type_errors/cross_not_vec3.glsl");
}

#[test]
fn test_builtin_wrong_arg_count_error() {
    test_file("type_errors/builtin_wrong_arg_count.glsl");
}

#[test]
fn test_function_return_type_mismatch_error() {
    test_file("type_errors/function_return_type_mismatch.glsl");
}

#[test]
fn test_function_wrong_arg_count_error() {
    test_file("type_errors/function_wrong_arg_count.glsl");
}

#[test]
fn test_function_wrong_arg_type_error() {
    test_file("type_errors/function_wrong_arg_type.glsl");
}

#[test]
fn test_parse_error() {
    test_file("type_errors/parse_error.glsl");
}

#[test]
fn test_undefined_variable_error() {
    test_file("type_errors/undefined_variable.glsl");
}

#[test]
fn test_undefined_function_error() {
    test_file("type_errors/undefined_function.glsl");
}

#[test]
fn test_no_main_function_error() {
    test_file("type_errors/no_main_function.glsl");
}

#[test]
fn test_unsupported_type_error() {
    test_file("type_errors/unsupported_type.glsl");
}

#[test]
fn test_invalid_vector_constructor_error() {
    test_file("type_errors/invalid_vector_constructor.glsl");
}

#[test]
fn test_return_type_mismatch_error() {
    test_file("type_errors/return_type_mismatch.glsl");
}

#[test]
fn test_builtin_wrong_arg_type_error() {
    test_file("type_errors/builtin_wrong_arg_type.glsl");
}

#[test]
fn test_assignment_lhs_error() {
    test_file("type_errors/assignment_lhs_error.glsl");
}

#[test]
fn test_swizzle_invalid_component_error() {
    test_file("type_errors/swizzle_invalid_component.glsl");
}

#[test]
fn test_swizzle_assign_duplicate_error() {
    test_file("type_errors/swizzle_assign_duplicate.glsl");
}

#[test]
fn test_trig_wrong_arg_type_error() {
    test_file("type_errors/trig_wrong_arg_type.glsl");
}

#[test]
fn test_trig_wrong_arg_count_error() {
    test_file("type_errors/trig_wrong_arg_count.glsl");
}

#[test]
fn test_atan2_type_mismatch_error() {
    test_file("type_errors/atan2_type_mismatch.glsl");
}

// Type error tests (previously commented out)
// TODO: Fix error test infrastructure to properly validate error messages
// #[test]
// fn test_mix_wrong_arg_count() {
//     test_file("type_errors/mix_wrong_arg_count.glsl");
// }
//
// #[test]
// fn test_mix_int_args() {
//     test_file("type_errors/mix_int_args.glsl");
// }
//
// #[test]
// fn test_step_wrong_arg_count() {
//     test_file("type_errors/step_wrong_arg_count.glsl");
// }
//
// #[test]
// fn test_smoothstep_wrong_arg_count() {
//     test_file("type_errors/smoothstep_wrong_arg_count.glsl");
// }
//
// #[test]
// fn test_fract_int_arg() {
//     test_file("type_errors/fract_int_arg.glsl");
// }
//
// #[test]
// fn test_mod_wrong_arg_count() {
//     test_file("type_errors/mod_wrong_arg_count.glsl");
// }
//
// #[test]
// fn test_sign_wrong_arg_count() {
//     test_file("type_errors/sign_wrong_arg_count.glsl");
// }
