//! Unit tests for GlslValue parsing and matrix handling

use lp_glsl::GlslValue;

#[test]
fn test_parse_mat2_from_column_vectors() {
    // mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))
    // Column 0: [1.0, 2.0]
    // Column 1: [3.0, 4.0]
    // Storage (column-major): [1.0, 2.0, 3.0, 4.0]
    // Internal representation: [[1.0, 2.0], [3.0, 4.0]] (column-major)
    let result = GlslValue::parse("mat2(vec2(1.0, 2.0), vec2(3.0, 4.0))").unwrap();
    match result {
        GlslValue::Mat2x2(m) => {
            // m[col][row] format
            // Column 0: [1.0, 2.0] (col0_row0, col0_row1)
            // Column 1: [3.0, 4.0] (col1_row0, col1_row1)
            assert_eq!(m[0][0], 1.0); // col0_row0
            assert_eq!(m[0][1], 2.0); // col0_row1
            assert_eq!(m[1][0], 3.0); // col1_row0
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
    let result =
        GlslValue::parse("mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0))")
            .unwrap();
    match result {
        GlslValue::Mat3x3(m) => {
            // m[col][row] format
            // Column 0: [1.0, 2.0, 3.0]
            assert_eq!(m[0][0], 1.0); // col0_row0
            assert_eq!(m[0][1], 2.0); // col0_row1
            assert_eq!(m[0][2], 3.0); // col0_row2
            // Column 1: [4.0, 5.0, 6.0]
            assert_eq!(m[1][0], 4.0); // col1_row0
            assert_eq!(m[1][1], 5.0); // col1_row1
            assert_eq!(m[1][2], 6.0); // col1_row2
            // Column 2: [7.0, 8.0, 9.0]
            assert_eq!(m[2][0], 7.0); // col2_row0
            assert_eq!(m[2][1], 8.0); // col2_row1
            assert_eq!(m[2][2], 9.0); // col2_row2
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
            // m[col][row] format
            // Column 0: [1.0, 0.0, 0.0, 0.0]
            assert_eq!(m[0][0], 1.0); // col0_row0
            assert_eq!(m[0][1], 0.0); // col0_row1
            assert_eq!(m[0][2], 0.0); // col0_row2
            assert_eq!(m[0][3], 0.0); // col0_row3
            // Column 1: [0.0, 1.0, 0.0, 0.0]
            assert_eq!(m[1][0], 0.0); // col1_row0
            assert_eq!(m[1][1], 1.0); // col1_row1
            assert_eq!(m[1][2], 0.0); // col1_row2
            assert_eq!(m[1][3], 0.0); // col1_row3
            // Column 2: [0.0, 0.0, 1.0, 0.0]
            assert_eq!(m[2][0], 0.0); // col2_row0
            assert_eq!(m[2][1], 0.0); // col2_row1
            assert_eq!(m[2][2], 1.0); // col2_row2
            assert_eq!(m[2][3], 0.0); // col2_row3
            // Column 3: [0.0, 0.0, 0.0, 1.0]
            assert_eq!(m[3][0], 0.0); // col3_row0
            assert_eq!(m[3][1], 0.0); // col3_row1
            assert_eq!(m[3][2], 0.0); // col3_row2
            assert_eq!(m[3][3], 1.0); // col3_row3
        }
        _ => panic!("Expected Mat4x4"),
    }
}
