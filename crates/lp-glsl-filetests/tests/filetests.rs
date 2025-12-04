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

// Builtin function tests
#[test]
fn test_mix_scalar() {
    test_file("builtins/mix_scalar.glsl");
}

#[test]
fn test_mix_vec3() {
    test_file("builtins/mix_vec3.glsl");
}

#[test]
fn test_mix_vec3_scalar_a() {
    test_file("builtins/mix_vec3_scalar_a.glsl");
}

#[test]
fn test_step_scalar() {
    test_file("builtins/step_scalar.glsl");
}

#[test]
fn test_step_vec3() {
    test_file("builtins/step_vec3.glsl");
}

#[test]
fn test_step_scalar_edge() {
    test_file("builtins/step_scalar_edge.glsl");
}

#[test]
fn test_smoothstep_scalar() {
    test_file("builtins/smoothstep_scalar.glsl");
}

#[test]
fn test_smoothstep_vec3() {
    test_file("builtins/smoothstep_vec3.glsl");
}

#[test]
fn test_smoothstep_scalar_edges() {
    test_file("builtins/smoothstep_scalar_edges.glsl");
}

#[test]
fn test_fract_scalar() {
    test_file("builtins/fract_scalar.glsl");
}

#[test]
fn test_fract_vec2() {
    test_file("builtins/fract_vec2.glsl");
}

#[test]
fn test_mod_scalar() {
    test_file("builtins/mod_scalar.glsl");
}

#[test]
fn test_mod_vec3() {
    test_file("builtins/mod_vec3.glsl");
}

#[test]
fn test_mod_vec3_scalar() {
    test_file("builtins/mod_vec3_scalar.glsl");
}

#[test]
fn test_sign_float() {
    test_file("builtins/sign_float.glsl");
}

#[test]
fn test_sign_int() {
    test_file("builtins/sign_int.glsl");
}

// Type error tests
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

