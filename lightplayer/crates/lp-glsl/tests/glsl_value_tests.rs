//! Unit tests for GlslValue parsing and comparison

use lp_glsl::GlslValue;
use lp_glsl::parse_program_with_registry;
use lp_glsl::infer_expr_type_in_context;
use lp_glsl::semantic::types::Type;

#[test]
fn test_glsl_value_parse_integers() {
    // Valid integers
    assert!(GlslValue::parse("0").unwrap().eq(&GlslValue::I32(0)));
    assert!(GlslValue::parse("42").unwrap().eq(&GlslValue::I32(42)));
    assert!(GlslValue::parse("-1").unwrap().eq(&GlslValue::I32(-1)));
}

#[test]
fn test_glsl_value_parse_floats() {
    // Valid floats
    assert!(GlslValue::parse("0.0").unwrap().eq(&GlslValue::F32(0.0)));
    assert!(GlslValue::parse("1.5").unwrap().eq(&GlslValue::F32(1.5)));
    assert!(GlslValue::parse("-3.14").unwrap().eq(&GlslValue::F32(-3.14)));
}

#[test]
fn test_glsl_value_parse_booleans() {
    // Valid booleans
    assert!(GlslValue::parse("true").unwrap().eq(&GlslValue::Bool(true)));
    assert!(GlslValue::parse("false").unwrap().eq(&GlslValue::Bool(false)));
}

#[test]
fn test_glsl_value_parse_invalid() {
    // Invalid literals
    assert!(GlslValue::parse("not_a_literal").is_err());
    assert!(GlslValue::parse("x + y").is_err());
    assert!(GlslValue::parse("add(1, 2)").is_err());
}

#[test]
fn test_glsl_value_eq_integers() {
    assert!(GlslValue::I32(42).eq(&GlslValue::I32(42)));
    assert!(!GlslValue::I32(42).eq(&GlslValue::I32(43)));
}

#[test]
fn test_glsl_value_eq_floats() {
    assert!(GlslValue::F32(1.0).eq(&GlslValue::F32(1.0)));
    assert!(!GlslValue::F32(1.0).eq(&GlslValue::F32(1.0001)));
}

#[test]
fn test_glsl_value_eq_booleans() {
    assert!(GlslValue::Bool(true).eq(&GlslValue::Bool(true)));
    assert!(!GlslValue::Bool(true).eq(&GlslValue::Bool(false)));
}

#[test]
fn test_glsl_value_eq_type_mismatch() {
    assert!(!GlslValue::I32(42).eq(&GlslValue::F32(42.0)));
}

#[test]
fn test_glsl_value_eq_vectors() {
    assert!(GlslValue::Vec2([1.0, 2.0]).eq(&GlslValue::Vec2([1.0, 2.0])));
    assert!(!GlslValue::Vec2([1.0, 2.0]).eq(&GlslValue::Vec2([1.0, 3.0])));
}

#[test]
fn test_glsl_value_eq_matrices() {
    let mat1 = GlslValue::Mat2x2([[1.0, 2.0], [3.0, 4.0]]);
    let mat2 = GlslValue::Mat2x2([[1.0, 2.0], [3.0, 4.0]]);
    let mat3 = GlslValue::Mat2x2([[1.0, 2.0], [3.0, 5.0]]);
    assert!(mat1.eq(&mat2));
    assert!(!mat1.eq(&mat3));
}

#[test]
fn test_glsl_value_approx_eq_integers() {
    // Integers use exact equality
    assert!(GlslValue::I32(42).approx_eq(&GlslValue::I32(42), 0.1));
    assert!(!GlslValue::I32(42).approx_eq(&GlslValue::I32(43), 0.1));
}

#[test]
fn test_glsl_value_approx_eq_floats() {
    // Floats within tolerance
    assert!(GlslValue::F32(1.0).approx_eq(&GlslValue::F32(1.00005), 0.0001));
    // Floats outside tolerance
    assert!(!GlslValue::F32(1.0).approx_eq(&GlslValue::F32(1.0002), 0.0001));
}

#[test]
fn test_glsl_value_approx_eq_booleans() {
    // Booleans use exact equality
    assert!(GlslValue::Bool(true).approx_eq(&GlslValue::Bool(true), 0.1));
    assert!(!GlslValue::Bool(true).approx_eq(&GlslValue::Bool(false), 0.1));
}

#[test]
fn test_glsl_value_approx_eq_vectors() {
    // Vectors within tolerance
    assert!(GlslValue::Vec2([1.0, 2.0]).approx_eq(&GlslValue::Vec2([1.00005, 2.00005]), 0.0001));
    // Vectors outside tolerance
    assert!(!GlslValue::Vec2([1.0, 2.0]).approx_eq(&GlslValue::Vec2([1.0002, 2.0]), 0.0001));
}

#[test]
fn test_glsl_value_approx_eq_matrices() {
    let mat1 = GlslValue::Mat2x2([[1.0, 2.0], [3.0, 4.0]]);
    let mat2 = GlslValue::Mat2x2([[1.00005, 2.0], [3.0, 4.0]]);
    let mat3 = GlslValue::Mat2x2([[1.0002, 2.0], [3.0, 4.0]]);
    assert!(mat1.approx_eq(&mat2, 0.0001));
    assert!(!mat1.approx_eq(&mat3, 0.0001));
}

#[test]
fn test_glsl_value_approx_eq_default() {
    // Within default tolerance (1e-4)
    assert!(GlslValue::F32(1.0).approx_eq_default(&GlslValue::F32(1.00005)));
    // Outside default tolerance
    assert!(!GlslValue::F32(1.0).approx_eq_default(&GlslValue::F32(1.0002)));
}

#[test]
fn test_parse_program_with_registry_simple() {
    let source = r#"
        int add_int(int a, int b) {
            return a + b;
        }
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    assert!(registry.lookup_function("add_int", &[Type::Int, Type::Int]).is_ok());
}

#[test]
fn test_parse_program_with_registry_multiple() {
    let source = r#"
        int add_int(int a, int b) {
            return a + b;
        }
        float add_float(float a, float b) {
            return a + b;
        }
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    assert!(registry.lookup_function("add_int", &[Type::Int, Type::Int]).is_ok());
    assert!(registry.lookup_function("add_float", &[Type::Float, Type::Float]).is_ok());
}

#[test]
fn test_parse_program_with_registry_empty() {
    let source = r#"
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    // Should have empty registry (only main function, no user functions)
    // Main is not in the registry, only user functions
    assert!(registry.lookup_function("add_int", &[Type::Int, Type::Int]).is_err());
}

#[test]
fn test_infer_expr_type_in_context_literals() {
    let source = r#"
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    
    // Literal expressions
    assert_eq!(infer_expr_type_in_context("42", &registry).unwrap(), Type::Int);
    assert_eq!(infer_expr_type_in_context("3.14", &registry).unwrap(), Type::Float);
}

#[test]
fn test_infer_expr_type_in_context_function_calls() {
    let source = r#"
        int add_int(int a, int b) {
            return a + b;
        }
        float add_float(float a, float b) {
            return a + b;
        }
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    
    // Function calls
    assert_eq!(infer_expr_type_in_context("add_int(1, 2)", &registry).unwrap(), Type::Int);
    assert_eq!(infer_expr_type_in_context("add_float(0.0, 0.0)", &registry).unwrap(), Type::Float);
}

#[test]
fn test_infer_expr_type_in_context_unknown_function() {
    let source = r#"
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    
    // Unknown function should error
    assert!(infer_expr_type_in_context("unknown_func(1, 2)", &registry).is_err());
}

#[test]
fn test_infer_expr_type_in_context_invalid_expression() {
    let source = r#"
        int main() {
            return 0;
        }
    "#;
    
    let registry = parse_program_with_registry(source).unwrap();
    
    // Invalid expression should error
    assert!(infer_expr_type_in_context("not a valid expression", &registry).is_err());
}




