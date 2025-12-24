//! Integration tests for matrix operations (parse → compile → execute → verify)

use lp_glsl::{DecimalFormat, GlslOptions, RunMode, GlslValue, glsl_jit, execute_main};

#[test]
fn test_mat2_constructor_round_trip() {
    // Parse mat2 constructor → compile → execute → verify output matches input
    let source = r#"
mat2 main() {
    return mat2(vec2(1.0, 2.0), vec2(3.0, 4.0));
}
"#;
    
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    
    let mut executable = glsl_jit(source, options).unwrap();
    let result = execute_main(&mut *executable).unwrap();
    
    match result {
        GlslValue::Mat2x2(m) => {
            // Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
            // m[col][row] format
            assert_eq!(m[0][0], 1.0); // col0_row0
            assert_eq!(m[0][1], 2.0); // col0_row1
            assert_eq!(m[1][0], 3.0); // col1_row0
            assert_eq!(m[1][1], 4.0); // col1_row1
        }
        _ => panic!("Expected Mat2x2"),
    }
}

#[test]
fn test_mat2_multiplication() {
    // Test matrix multiplication: mat2(vec2(1,2), vec2(3,4)) * mat2(vec2(5,6), vec2(7,8))
    // Expected result: mat2(vec2(23, 31), vec2(34, 46))
    let source = r#"
mat2 main() {
    mat2 a = mat2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    mat2 b = mat2(vec2(5.0, 6.0), vec2(7.0, 8.0));
    return a * b;
}
"#;
    
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    
    let mut executable = glsl_jit(source, options).unwrap();
    let result = execute_main(&mut *executable).unwrap();
    
    match result {
        GlslValue::Mat2x2(m) => {
            // Expected: Column 0: [23.0, 34.0], Column 1: [31.0, 46.0]
            // m[col][row] format
            assert_eq!(m[0][0], 23.0); // col0_row0
            assert_eq!(m[0][1], 34.0); // col0_row1
            assert_eq!(m[1][0], 31.0); // col1_row0
            assert_eq!(m[1][1], 46.0); // col1_row1
        }
        _ => panic!("Expected Mat2x2"),
    }
}

#[test]
fn test_mat2_indexing_column() {
    // Test matrix indexing: mat[0] returns first column
    let source = r#"
vec2 main() {
    mat2 m = mat2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    return m[0];
}
"#;
    
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    
    let mut executable = glsl_jit(source, options).unwrap();
    let result = execute_main(&mut *executable).unwrap();
    
    match result {
        GlslValue::Vec2(v) => {
            // First column should be [1.0, 2.0]
            assert_eq!(v[0], 1.0);
            assert_eq!(v[1], 2.0);
        }
        _ => panic!("Expected Vec2"),
    }
}

#[test]
fn test_mat2_indexing_element() {
    // Test matrix element access: mat[0][0] returns first element
    let source = r#"
float main() {
    mat2 m = mat2(vec2(1.0, 2.0), vec2(3.0, 4.0));
    return m[0][0];
}
"#;
    
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    
    let mut executable = glsl_jit(source, options).unwrap();
    let result = execute_main(&mut *executable).unwrap();
    
    match result {
        GlslValue::F32(f) => {
            // First element (col0_row0) should be 1.0
            assert_eq!(f, 1.0);
        }
        _ => panic!("Expected F32"),
    }
}

#[test]
fn test_mat3_multiplication() {
    // Test mat3 multiplication with identity matrices
    let source = r#"
mat3 main() {
    mat3 a = mat3(vec3(1.0, 0.0, 0.0), vec3(0.0, 1.0, 0.0), vec3(0.0, 0.0, 1.0));
    mat3 b = mat3(vec3(2.0, 0.0, 0.0), vec3(0.0, 2.0, 0.0), vec3(0.0, 0.0, 2.0));
    return a * b;
}
"#;
    
    let options = GlslOptions {
        run_mode: RunMode::HostJit,
        decimal_format: DecimalFormat::Float,
    };
    
    let mut executable = glsl_jit(source, options).unwrap();
    let result = execute_main(&mut *executable).unwrap();
    
    match result {
        GlslValue::Mat3x3(m) => {
            // Result should be: mat3(vec3(2.0, 0.0, 0.0), vec3(0.0, 2.0, 0.0), vec3(0.0, 0.0, 2.0))
            // Column 0: [2.0, 0.0, 0.0]
            assert_eq!(m[0][0], 2.0);
            assert_eq!(m[1][0], 0.0);
            assert_eq!(m[2][0], 0.0);
            // Column 1: [0.0, 2.0, 0.0]
            assert_eq!(m[0][1], 0.0);
            assert_eq!(m[1][1], 2.0);
            assert_eq!(m[2][1], 0.0);
            // Column 2: [0.0, 0.0, 2.0]
            assert_eq!(m[0][2], 0.0);
            assert_eq!(m[1][2], 0.0);
            assert_eq!(m[2][2], 2.0);
        }
        _ => panic!("Expected Mat3x3"),
    }
}

