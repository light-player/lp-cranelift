//! Integration test that runs all filetests

use std::path::PathBuf;

fn test_file(path: &str) {
    let mut full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    full_path.push("filetests");
    full_path.push(path);
    
    lp_glsl::testing::run_filetest(&full_path).unwrap();
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

