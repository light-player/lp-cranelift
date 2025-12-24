//! Unit tests for GlslValue parsing and matrix handling

use lp_glsl::GlslValue;

#[test]
fn test_parse_mat2_from_column_vectors() {
    // mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))
    // Column 0: [1.0, 2.0]
    // Column 1: [3.0, 4.0]
    // Storage (column-major): [1.0, 2.0, 3.0, 4.0]
    // Internal representation: [[1.0, 3.0], [2.0, 4.0]] (row-major view of column-major storage)
    let result = GlslValue::parse("mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))").unwrap();
    match result {
        GlslValue::Mat2x2(m) => {
            // m[row][col] format
            // Row 0: [1.0, 3.0] (col0_row0, col1_row0)
            // Row 1: [2.0, 4.0] (col0_row1, col1_row1)
            assert_eq!(m[0][0], 1.0); // col0_row0
            assert_eq!(m[1][0], 2.0); // col0_row1
            assert_eq!(m[0][1], 3.0); // col1_row0
            assert_eq!(m[1][1], 4.0); // col1_row1
        }
        _ => panic!("Expected Mat2x2"),
    }
}

// Note: Scalar matrix constructors (mat2(1.0, 2.0, 3.0, 4.0)) are not currently
// supported by GlslValue::parse() which only handles column vector constructors.
// This is acceptable as column vector constructors are the primary form.
// Scalar constructors are handled in codegen (constructor.rs) but not in the
// test value parser.

#[test]
fn test_parse_mat3_from_column_vectors() {
    // mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0))
    let result = GlslValue::parse("mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0))").unwrap();
    match result {
        GlslValue::Mat3x3(m) => {
            // Column 0: [1.0, 2.0, 3.0]
            assert_eq!(m[0][0], 1.0);
            assert_eq!(m[1][0], 2.0);
            assert_eq!(m[2][0], 3.0);
            // Column 1: [4.0, 5.0, 6.0]
            assert_eq!(m[0][1], 4.0);
            assert_eq!(m[1][1], 5.0);
            assert_eq!(m[2][1], 6.0);
            // Column 2: [7.0, 8.0, 9.0]
            assert_eq!(m[0][2], 7.0);
            assert_eq!(m[1][2], 8.0);
            assert_eq!(m[2][2], 9.0);
        }
        _ => panic!("Expected Mat3x3"),
    }
}

#[test]
fn test_parse_mat4_from_column_vectors() {
    // mat4 with identity-like pattern
    let result = GlslValue::parse("mat4(vec4(1.0, 0.0, 0.0, 0.0), vec4(0.0, 1.0, 0.0, 0.0), vec4(0.0, 0.0, 1.0, 0.0), vec4(0.0, 0.0, 0.0, 1.0))").unwrap();
    match result {
        GlslValue::Mat4x4(m) => {
            // Column 0: [1.0, 0.0, 0.0, 0.0]
            assert_eq!(m[0][0], 1.0);
            assert_eq!(m[1][0], 0.0);
            assert_eq!(m[2][0], 0.0);
            assert_eq!(m[3][0], 0.0);
            // Column 1: [0.0, 1.0, 0.0, 0.0]
            assert_eq!(m[0][1], 0.0);
            assert_eq!(m[1][1], 1.0);
            assert_eq!(m[2][1], 0.0);
            assert_eq!(m[3][1], 0.0);
            // Column 2: [0.0, 0.0, 1.0, 0.0]
            assert_eq!(m[0][2], 0.0);
            assert_eq!(m[1][2], 0.0);
            assert_eq!(m[2][2], 1.0);
            assert_eq!(m[3][2], 0.0);
            // Column 3: [0.0, 0.0, 0.0, 1.0]
            assert_eq!(m[0][3], 0.0);
            assert_eq!(m[1][3], 0.0);
            assert_eq!(m[2][3], 0.0);
            assert_eq!(m[3][3], 1.0);
        }
        _ => panic!("Expected Mat4x4"),
    }
}

