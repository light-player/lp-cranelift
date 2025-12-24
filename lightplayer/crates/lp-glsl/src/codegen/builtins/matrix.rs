//! Matrix built-in functions

use crate::codegen::context::CodegenContext;
use crate::error::{ErrorCode, GlslError};
use crate::semantic::types::Type;
use cranelift_codegen::ir::{InstBuilder, Value};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

#[allow(non_snake_case)]
impl<'a> CodegenContext<'a> {
    /// Component-wise matrix multiply: result[i][j] = x[i][j] * y[i][j]
    pub fn builtin_matrixCompMult(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, y_ty) = &args[1];

        if x_ty != y_ty || !x_ty.is_matrix() {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "matrixCompMult() requires two matrices of the same type",
            ));
        }

        let mut result_vals = Vec::new();
        for (x_val, y_val) in x_vals.iter().zip(y_vals.iter()) {
            result_vals.push(self.builder.ins().fmul(*x_val, *y_val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// Outer product: vec1 × vec2 → matrix
    /// For vec3 × vec3, returns mat3 where result[i][j] = vec1[i] * vec2[j]
    pub fn builtin_outerProduct(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (vec1_vals, vec1_ty) = &args[0];
        let (vec2_vals, vec2_ty) = &args[1];

        if !vec1_ty.is_vector() || !vec2_ty.is_vector() {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "outerProduct() requires two vectors",
            ));
        }

        let vec1_size = vec1_vals.len();
        let vec2_size = vec2_vals.len();

        // Determine result matrix type based on vector sizes
        let result_ty = match (vec1_size, vec2_size) {
            (2, 2) => Type::Mat2,
            (3, 3) => Type::Mat3,
            (4, 4) => Type::Mat4,
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0104,
                    format!(
                        "outerProduct() requires matching vector sizes (got {} and {})",
                        vec1_size, vec2_size
                    ),
                ));
            }
        };

        // Compute outer product: result[i][j] = vec1[i] * vec2[j]
        // Result matrix is stored column-major
        let mut result_vals = Vec::new();
        for j in 0..vec2_size {
            for i in 0..vec1_size {
                let product = self.builder.ins().fmul(vec1_vals[i], vec2_vals[j]);
                result_vals.push(product);
            }
        }

        Ok((result_vals, result_ty))
    }

    /// Transpose matrix: swap rows and columns
    pub fn builtin_transpose(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (m_vals, m_ty) = &args[0];

        if !m_ty.is_matrix() {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "transpose() requires a matrix",
            ));
        }

        let (rows, cols) = m_ty.matrix_dims().unwrap();

        // Transpose: result[col][row] = m[row][col]
        // Input is column-major: m_vals[col * rows + row] = m[col][row]
        // Output should be column-major: result_vals[col * rows + row] = result[col][row] = m[row][col]
        // m[row][col] = m_vals[row * rows + col] (since row becomes column index)
        // So: result_vals[result_col * rows + result_row] = m_vals[result_row * rows + result_col]
        let mut result_vals = Vec::new();
        for result_col in 0..rows {
            // Transposed matrix has rows columns
            for result_row in 0..cols {
                // Transposed matrix has cols rows
                // result[result_col][result_row] = m[result_row][result_col]
                // m[result_row][result_col] is at: result_row * rows + result_col
                let old_idx = result_row * rows + result_col;
                result_vals.push(m_vals[old_idx]);
            }
        }

        // Result type is the same (square matrices)
        Ok((result_vals, m_ty.clone()))
    }

    /// Compute matrix determinant
    pub fn builtin_determinant(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (m_vals, m_ty) = &args[0];

        if !m_ty.is_matrix() {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "determinant() requires a matrix",
            ));
        }

        let (rows, cols) = m_ty.matrix_dims().unwrap();
        if rows != cols {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "determinant() only supported for square matrices",
            ));
        }

        // Helper to get element at (row, col) from column-major storage
        let get = |row: usize, col: usize| -> Value { m_vals[col * rows + row] };

        let det = match rows {
            2 => {
                // det = a*d - b*c
                // Matrix: [a b]  stored as [a, c, b, d] (col-major)
                //         [c d]
                let a = get(0, 0);
                let b = get(0, 1);
                let c = get(1, 0);
                let d = get(1, 1);
                let ad = self.builder.ins().fmul(a, d);
                let bc = self.builder.ins().fmul(b, c);
                self.builder.ins().fsub(ad, bc)
            }
            3 => {
                // Sarrus rule for 3x3
                // det = a(ei - fh) - b(di - fg) + c(dh - eg)
                let a = get(0, 0);
                let b = get(0, 1);
                let c = get(0, 2);
                let d = get(1, 0);
                let e = get(1, 1);
                let f = get(1, 2);
                let g = get(2, 0);
                let h = get(2, 1);
                let i = get(2, 2);

                let ei = self.builder.ins().fmul(e, i);
                let fh = self.builder.ins().fmul(f, h);
                let ei_minus_fh = self.builder.ins().fsub(ei, fh);
                let term1 = self.builder.ins().fmul(a, ei_minus_fh);

                let di = self.builder.ins().fmul(d, i);
                let fg = self.builder.ins().fmul(f, g);
                let di_minus_fg = self.builder.ins().fsub(di, fg);
                let term2 = self.builder.ins().fmul(b, di_minus_fg);

                let dh = self.builder.ins().fmul(d, h);
                let eg = self.builder.ins().fmul(e, g);
                let dh_minus_eg = self.builder.ins().fsub(dh, eg);
                let term3 = self.builder.ins().fmul(c, dh_minus_eg);

                let term1_minus_term2 = self.builder.ins().fsub(term1, term2);
                self.builder.ins().fadd(term1_minus_term2, term3)
            }
            4 => {
                // Cofactor expansion for 4x4 (using first row)
                // This is complex, so we'll use a simpler approach: compute via minors
                // For now, return an error and implement later if needed
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "4x4 determinant not yet implemented",
                ));
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0104,
                    format!(
                        "determinant() not supported for {}-dimensional matrices",
                        rows
                    ),
                ));
            }
        };

        Ok((vec![det], Type::Float))
    }

    /// Compute matrix inverse
    pub fn builtin_inverse(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (m_vals, m_ty) = &args[0];

        if !m_ty.is_matrix() {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "inverse() requires a matrix",
            ));
        }

        let (rows, cols) = m_ty.matrix_dims().unwrap();
        if rows != cols {
            return Err(GlslError::new(
                ErrorCode::E0104,
                "inverse() only supported for square matrices",
            ));
        }

        // Helper to get element at (row, col) from column-major storage
        let get = |row: usize, col: usize| -> Value { m_vals[col * rows + row] };

        match rows {
            2 => {
                // Inverse of 2x2: (1/det) * [d -b]
                //                              [-c a]
                // Matrix: [a b]  stored as [a, c, b, d] (col-major)
                //         [c d]
                let a = get(0, 0);
                let b = get(0, 1);
                let c = get(1, 0);
                let d = get(1, 1);

                // Compute determinant
                let ad = self.builder.ins().fmul(a, d);
                let bc = self.builder.ins().fmul(b, c);
                let det = self.builder.ins().fsub(ad, bc);

                // Compute 1/det
                let one = self.builder.ins().f32const(1.0);
                let inv_det = self.builder.ins().fdiv(one, det);

                // Compute inverse elements (stored column-major)
                // result[0][0] = d * inv_det
                // result[1][0] = -c * inv_det
                // result[0][1] = -b * inv_det
                // result[1][1] = a * inv_det
                let zero = self.builder.ins().f32const(0.0);
                let minus_c = self.builder.ins().fsub(zero, c);
                let minus_b = self.builder.ins().fsub(zero, b);

                let mut result_vals = Vec::new();
                result_vals.push(self.builder.ins().fmul(d, inv_det)); // result[0][0]
                result_vals.push(self.builder.ins().fmul(minus_c, inv_det)); // result[1][0]
                result_vals.push(self.builder.ins().fmul(minus_b, inv_det)); // result[0][1]
                result_vals.push(self.builder.ins().fmul(a, inv_det)); // result[1][1]

                Ok((result_vals, m_ty.clone()))
            }
            3 => {
                // For 3x3, use adjugate/determinant method
                // This is complex, so for now return error
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "3x3 matrix inverse not yet implemented",
                ));
            }
            4 => {
                return Err(GlslError::new(
                    ErrorCode::E0400,
                    "4x4 matrix inverse not yet implemented",
                ));
            }
            _ => {
                return Err(GlslError::new(
                    ErrorCode::E0104,
                    format!("inverse() not supported for {}-dimensional matrices", rows),
                ));
            }
        }
    }
}

