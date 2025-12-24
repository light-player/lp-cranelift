//! Unit tests for test_utils.rs matrix conversion logic

use lp_glsl::GlslValue;

#[test]
fn test_flat_array_to_mat2x2_conversion() {
    // Test the conversion logic from test_utils.rs line 71
    // Flat array from emulator (column-major): [col0_row0, col0_row1, col1_row0, col1_row1]
    // For mat2(vec2(1.0, 2.0), vec2(3.0, 4.0)):
    // Storage: [1.0, 2.0, 3.0, 4.0]
    // Conversion: [[v[0], v[2]], [v[1], v[3]]] = [[1.0, 3.0], [2.0, 4.0]]
    
    let flat_array = vec![1.0, 2.0, 3.0, 4.0];
    
    // Simulate the conversion from test_utils.rs
    let mat = GlslValue::Mat2x2([
        [flat_array[0], flat_array[2]],  // [1.0, 3.0] - row 0
        [flat_array[1], flat_array[3]],    // [2.0, 4.0] - row 1
    ]);
    
    // Verify the matrix represents the correct values
    // Column 0 should be [1.0, 2.0], Column 1 should be [3.0, 4.0]
    match mat {
        GlslValue::Mat2x2(m) => {
            // m[row][col] format
            // Column 0: [m[0][0], m[1][0]] = [1.0, 2.0] ✓
            assert_eq!(m[0][0], 1.0); // col0_row0
            assert_eq!(m[1][0], 2.0); // col0_row1
            // Column 1: [m[0][1], m[1][1]] = [3.0, 4.0] ✓
            assert_eq!(m[0][1], 3.0); // col1_row0
            assert_eq!(m[1][1], 4.0); // col1_row1
        }
        _ => panic!("Expected Mat2x2"),
    }
}

#[test]
fn test_flat_array_to_mat3x3_conversion() {
    // Test the conversion logic from test_utils.rs line 78
    // Flat array (column-major): [col0_row0, col0_row1, col0_row2, col1_row0, col1_row1, col1_row2, col2_row0, col2_row1, col2_row2]
    // For mat3(vec3(1.0, 2.0, 3.0), vec3(4.0, 5.0, 6.0), vec3(7.0, 8.0, 9.0)):
    // Storage: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0]
    // Conversion: [[v[0], v[3], v[6]], [v[1], v[4], v[7]], [v[2], v[5], v[8]]]
    
    let flat_array = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
    
    // Simulate the conversion from test_utils.rs
    let mat = GlslValue::Mat3x3([
        [flat_array[0], flat_array[3], flat_array[6]],  // row 0
        [flat_array[1], flat_array[4], flat_array[7]],  // row 1
        [flat_array[2], flat_array[5], flat_array[8]],  // row 2
    ]);
    
    // Verify columns are correct
    match mat {
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
fn test_flat_array_to_mat4x4_conversion() {
    // Test the conversion logic from test_utils.rs lines 85-90
    // Flat array (column-major): 16 elements
    // Conversion pattern: [[v[0], v[4], v[8], v[12]], [v[1], v[5], v[9], v[13]], [v[2], v[6], v[10], v[14]], [v[3], v[7], v[11], v[15]]]
    
    // Identity matrix
    let flat_array = vec![
        1.0, 0.0, 0.0, 0.0,  // column 0
        0.0, 1.0, 0.0, 0.0,  // column 1
        0.0, 0.0, 1.0, 0.0,  // column 2
        0.0, 0.0, 0.0, 1.0,  // column 3
    ];
    
    // Simulate the conversion from test_utils.rs
    let mat = GlslValue::Mat4x4([
        [flat_array[0], flat_array[4], flat_array[8], flat_array[12]],   // row 0
        [flat_array[1], flat_array[5], flat_array[9], flat_array[13]], // row 1
        [flat_array[2], flat_array[6], flat_array[10], flat_array[14]], // row 2
        [flat_array[3], flat_array[7], flat_array[11], flat_array[15]], // row 3
    ]);
    
    // Verify columns are correct
    match mat {
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

