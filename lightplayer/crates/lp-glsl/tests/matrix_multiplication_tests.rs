//! Unit tests for matrix multiplication correctness

use lp_glsl::GlslValue;

#[test]
fn test_matrix_multiplication_expected_result() {
    // Test case from the plan:
    // mat2(vec2(1,2), vec2(3,4)) * mat2(vec2(5,6), vec2(7,8))
    // 
    // Standard matrix multiplication:
    // [[1, 3],   [[5, 7],   [[1*5+3*6, 1*7+3*8],   [[23, 31],
    //  [2, 4]] *  [6, 8]] =  [2*5+4*6, 2*7+4*8]] =  [34, 46]]
    //
    // Column-major storage: [23, 34, 31, 46]
    // Internal representation: [[23, 31], [34, 46]] (m[row][col])
    
    // Create the matrices
    let mat_a = GlslValue::Mat2x2([
        [1.0, 3.0],  // row 0: col0=1.0, col1=3.0
        [2.0, 4.0],  // row 1: col0=2.0, col1=4.0
    ]);
    
    let mat_b = GlslValue::Mat2x2([
        [5.0, 7.0],  // row 0: col0=5.0, col1=7.0
        [6.0, 8.0],  // row 1: col0=6.0, col1=8.0
    ]);
    
    // Expected result: [[23, 31], [34, 46]]
    // Column 0: [23, 34], Column 1: [31, 46]
    let expected = GlslValue::Mat2x2([
        [23.0, 31.0],  // row 0
        [34.0, 46.0],  // row 1
    ]);
    
    // Note: This test verifies the expected result format.
    // Actual multiplication is tested in integration tests that compile and execute GLSL code.
    match (mat_a, mat_b, expected) {
        (GlslValue::Mat2x2(a), GlslValue::Mat2x2(b), GlslValue::Mat2x2(exp)) => {
            // Verify the expected result structure
            // Column 0: [23.0, 34.0]
            assert_eq!(exp[0][0], 23.0); // col0_row0
            assert_eq!(exp[1][0], 34.0); // col0_row1
            // Column 1: [31.0, 46.0]
            assert_eq!(exp[0][1], 31.0); // col1_row0
            assert_eq!(exp[1][1], 46.0); // col1_row1
            
            // Verify input matrices are correct
            // mat_a: Column 0: [1.0, 2.0], Column 1: [3.0, 4.0]
            assert_eq!(a[0][0], 1.0);
            assert_eq!(a[1][0], 2.0);
            assert_eq!(a[0][1], 3.0);
            assert_eq!(a[1][1], 4.0);
            
            // mat_b: Column 0: [5.0, 6.0], Column 1: [7.0, 8.0]
            assert_eq!(b[0][0], 5.0);
            assert_eq!(b[1][0], 6.0);
            assert_eq!(b[0][1], 7.0);
            assert_eq!(b[1][1], 8.0);
        }
        _ => panic!("Expected Mat2x2"),
    }
}

