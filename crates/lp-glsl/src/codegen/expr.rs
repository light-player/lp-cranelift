use cranelift_codegen::ir::{condcodes::{IntCC, FloatCC}, types, InstBuilder, Value};
use glsl::syntax::Expr;

#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::{infer_binary_result_type, infer_unary_result_type, promote_numeric, check_assignment, check_vector_constructor_with_span, is_vector_type_name, is_matrix_type_name, check_matrix_constructor};
use crate::error::{ErrorCode, GlslError, extract_span_from_expr, extract_span_from_identifier, source_span_to_location};

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

#[cfg(feature = "std")]
use std::collections::HashSet;
#[cfg(not(feature = "std"))]
use hashbrown::HashSet;

/// Component naming sets for vector swizzles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NamingSet {
    XYZW,  // Position/generic: x, y, z, w
    RGBA,  // Color: r, g, b, a
    STPQ,  // Texture coordinates: s, t, p, q
}

impl<'a> CodegenContext<'a> {
    /// Translate expression and return values (vec for vectors, single element for scalars) and type
    pub fn translate_expr_typed(&mut self, expr: &Expr) -> Result<(Vec<Value>, GlslType), GlslError> {
        match expr {
            // Literals - scalars return single-element vec
            Expr::IntConst(n, _) => {
                let val = self.builder.ins().iconst(types::I32, *n as i64);
                Ok((vec![val], GlslType::Int))
            }

            Expr::FloatConst(f, _) => {
                let val = self.builder.ins().f32const(*f);
                Ok((vec![val], GlslType::Float))
            }

            Expr::BoolConst(b, _) => {
                let val = self.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
                Ok((vec![val], GlslType::Bool))
            }

            // Variable reference - returns all components
            Expr::Variable(ident, _span) => {
                let span = extract_span_from_identifier(ident);
                let vars = self
                    .lookup_variables(&ident.name)
                    .ok_or_else(|| {
                        let error = GlslError::undefined_variable(&ident.name)
                            .with_location(source_span_to_location(&span));
                        self.add_span_to_error(error, &span)
                    })?
                    .to_vec(); // Clone to avoid borrow issues
                let ty = self
                    .lookup_variable_type(&ident.name)
                    .ok_or_else(|| {
                        let error = GlslError::new(ErrorCode::E0400, format!("variable type not found for `{}` during codegen", ident.name))
                            .with_location(source_span_to_location(&span));
                        self.add_span_to_error(error, &span)
                    })?
                    .clone();
                
                let vals: Vec<Value> = vars.iter()
                    .map(|&v| self.builder.use_var(v))
                    .collect();
                
                Ok((vals, ty))
            }

            // Function calls - check if it's a type constructor or built-in
            Expr::FunCall(func_ident, args, span) => {
                // Extract identifier name from FunIdentifier enum
                let func_name = match func_ident {
                    glsl::syntax::FunIdentifier::Identifier(ident) => &ident.name,
                    _ => return Err(GlslError::new(ErrorCode::E0400, "complex function identifiers not yet supported")),
                };
                
                // Check if it's a type constructor
                if is_vector_type_name(func_name) {
                    return self.translate_vector_constructor(func_name, args, span.clone());
                }
                
                if is_matrix_type_name(func_name) {
                    return self.translate_matrix_constructor(func_name, args);
                }
                
                // Check if it's a built-in function
                if crate::semantic::builtins::is_builtin_function(func_name) {
                    return self.translate_builtin_call_expr(func_name, args, span.clone());
                }
                
                // User-defined function
                self.translate_user_function_call(func_name, args, span.clone())
            }

            // Binary operators - scalars, vectors, and matrices
            Expr::Binary(op, lhs, rhs, span) => {
                let (lhs_vals, lhs_ty) = self.translate_expr_typed(lhs)?;
                let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;

                // Check if it's matrix operation
                if lhs_ty.is_matrix() || rhs_ty.is_matrix() {
                    self.translate_matrix_binary_op(op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, span.clone())
                }
                // Check if it's vector operation
                else if lhs_ty.is_vector() || rhs_ty.is_vector() {
                    self.translate_vector_binary_op(op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, Some(span.clone()))
                } else {
                    // Scalar operation
                    let lhs_val = lhs_vals[0];
                    let rhs_val = rhs_vals[0];

                    // Infer result type and validate
                    let result_ty = infer_binary_result_type(op, &lhs_ty, &rhs_ty, span.clone())?;

                    // Check if operator is logical or comparison (skip promotion for these)
                    use glsl::syntax::BinaryOp::*;
                    let is_logical = matches!(op, And | Or | Xor);
                    let is_comparison = matches!(op, Equal | NonEqual | LT | GT | LTE | GTE);

                    let (lhs_val, rhs_val, operand_ty) = if is_logical {
                        // Logical operators: both operands must be Bool (validated above)
                        // Skip promotion - use Bool directly
                        (lhs_val, rhs_val, GlslType::Bool)
                    } else if is_comparison {
                        // Comparison operators: operands are numeric, may need promotion
                        // if different types (Int vs Float)
                        let common_ty = promote_numeric(&lhs_ty, &rhs_ty);
                        let lhs_val = self.coerce_to_type(lhs_val, &lhs_ty, &common_ty)?;
                        let rhs_val = self.coerce_to_type(rhs_val, &rhs_ty, &common_ty)?;
                        (lhs_val, rhs_val, common_ty)
                    } else {
                        // Arithmetic operators: promote to common type
                        let common_ty = promote_numeric(&lhs_ty, &rhs_ty);
                        let lhs_val = self.coerce_to_type(lhs_val, &lhs_ty, &common_ty)?;
                        let rhs_val = self.coerce_to_type(rhs_val, &rhs_ty, &common_ty)?;
                        (lhs_val, rhs_val, common_ty)
                    };

                    // Generate operation
                    let result_val = self.translate_scalar_binary_op(op, lhs_val, rhs_val, &operand_ty)?;
                    Ok((vec![result_val], result_ty))
                }
            }

            // Unary operators - scalars only for now
            Expr::Unary(op, expr, span) => {
                let (vals, ty) = self.translate_expr_typed(expr)?;
                
                if vals.len() != 1 {
                    return Err(GlslError::new(ErrorCode::E0400, "vector unary ops not yet implemented"));
                }

                let val = vals[0];
                let result_ty = infer_unary_result_type(op, &ty, span.clone())?;
                let result_val = self.translate_unary_op(op, val, &ty)?;
                Ok((vec![result_val], result_ty))
            }

            // Component access (field selection) - supports multi-component swizzles
            Expr::Dot(expr, field, dot_span) => {
                let (vals, ty) = self.translate_expr_typed(expr)?;
                
                if !ty.is_vector() {
                    let span = extract_span_from_expr(expr);
                    let error = GlslError::new(ErrorCode::E0112, format!("component access on non-vector type: {:?}", ty))
                        .with_location(source_span_to_location(&span));
                    return Err(self.add_span_to_error(error, &span));
                }

                // Use the span from the dot expression for error reporting
                let indices = Self::parse_vector_swizzle(&field.name, &ty, Some(dot_span.clone()))?;
                let base_ty = ty.vector_base_type().unwrap();
                
                if indices.len() == 1 {
                    // Single component: return scalar
                    Ok((vec![vals[indices[0]]], base_ty))
                } else {
                    // Multi-component: return vector
                    let result_vals: Vec<Value> = indices.iter()
                        .map(|&idx| vals[idx])
                        .collect();
                    let result_ty = GlslType::vector_type(&base_ty, indices.len())
                        .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("cannot create vector of size {}", indices.len())))?;
                    Ok((result_vals, result_ty))
                }
            }

            // Matrix/Array indexing: mat[i] returns column vector, mat[i][j] returns element
            Expr::Bracket(array_expr, array_spec, span) => {
                let (array_vals, array_ty) = self.translate_expr_typed(array_expr)?;
                
                if !array_ty.is_matrix() {
                    return Err(GlslError::new(ErrorCode::E0400, "indexing only supported for matrices")
                        .with_location(source_span_to_location(span)));
                }

                // Extract index expression from ArraySpecifier
                // ArraySpecifier contains dimensions field with a Vec of ArraySpecifierDimension
                // For matrix indexing mat[i], we need the first dimension's expression
                use glsl::syntax::ArraySpecifierDimension;
                if array_spec.dimensions.0.is_empty() {
                    return Err(GlslError::new(ErrorCode::E0400, "matrix indexing requires explicit index")
                        .with_location(source_span_to_location(span)));
                }
                let index_expr = match &array_spec.dimensions.0[0] {
                    ArraySpecifierDimension::ExplicitlySized(expr) => expr,
                    ArraySpecifierDimension::Unsized => {
                        return Err(GlslError::new(ErrorCode::E0400, "matrix indexing requires explicit index")
                            .with_location(source_span_to_location(span)));
                    }
                };

                // Evaluate index (must be int)
                let (_, index_ty) = self.translate_expr_typed(index_expr)?;
                if index_ty != GlslType::Int {
                    return Err(GlslError::new(ErrorCode::E0106, "matrix index must be int")
                        .with_location(source_span_to_location(span)));
                }

                let (rows, cols) = array_ty.matrix_dims().unwrap();
                let column_type = array_ty.matrix_column_type().unwrap();

                // Extract compile-time constant index
                // TODO: Support runtime indices
                let col_index = if let Expr::IntConst(n, _) = index_expr.as_ref() {
                    let n = *n as usize;
                    if n >= cols {
                        return Err(GlslError::new(ErrorCode::E0400, format!("matrix column index {} out of bounds (max {})", n, cols - 1))
                            .with_location(source_span_to_location(span)));
                    }
                    n
                } else {
                    return Err(GlslError::new(ErrorCode::E0400, "matrix indexing with variable index not yet implemented")
                        .with_location(source_span_to_location(span))
                        .with_note("only compile-time constant indices are supported"));
                };

                // Extract column elements
                // Matrix is stored column-major: [col0_row0, col0_row1, ..., col1_row0, ...]
                let mut column_vals = Vec::new();
                for row in 0..rows {
                    let idx = col_index * rows + row;
                    column_vals.push(array_vals[idx]);
                }

                Ok((column_vals, column_type))
            }

            // Assignment
            Expr::Assignment(lhs, op, rhs, _span) => self.translate_assignment_typed(lhs, op, rhs),

            _ => Err(GlslError::new(ErrorCode::E0400, format!("expression not supported yet: {:?}", expr))),
        }
    }

    /// Legacy wrapper for compatibility - returns just the first value (for scalars)
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<Value, GlslError> {
        let (vals, _ty) = self.translate_expr_typed(expr)?;
        vals.into_iter().next().ok_or_else(|| GlslError::new(ErrorCode::E0400, "expression produced no values"))
    }

    /// Translate scalar binary operation (works on single values)
    fn translate_scalar_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs: Value,
        rhs: Value,
        operand_ty: &GlslType,
    ) -> Result<Value, GlslError> {
        use glsl::syntax::BinaryOp::*;

        let val = match op {
            // Arithmetic operators - dispatch based on type
            Add => match operand_ty {
                GlslType::Int => self.builder.ins().iadd(lhs, rhs),
                GlslType::Float => self.builder.ins().fadd(lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("add not supported for {:?}", operand_ty))),
            },
            Sub => match operand_ty {
                GlslType::Int => self.builder.ins().isub(lhs, rhs),
                GlslType::Float => self.builder.ins().fsub(lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("sub not supported for {:?}", operand_ty))),
            },
            Mult => match operand_ty {
                GlslType::Int => self.builder.ins().imul(lhs, rhs),
                GlslType::Float => self.builder.ins().fmul(lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("mult not supported for {:?}", operand_ty))),
            },
            Div => match operand_ty {
                GlslType::Int => self.builder.ins().sdiv(lhs, rhs),
                GlslType::Float => self.builder.ins().fdiv(lhs, rhs),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("div not supported for {:?}", operand_ty))),
            },

            // Comparison operators - dispatch based on type
            // icmp/fcmp return I1, but GLSL bool is I8, so convert
            Equal => {
                let cmp_result = match operand_ty {
                    GlslType::Int => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                    GlslType::Float => self.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
                    _ => return Err(GlslError::new(ErrorCode::E0400, format!("equal not supported for {:?}", operand_ty))),
                };
                // Convert I1 to I8: select 1 if true, 0 if false
                let one = self.builder.ins().iconst(types::I8, 1);
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().select(cmp_result, one, zero)
            },
            NonEqual => {
                let cmp_result = match operand_ty {
                    GlslType::Int => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                    GlslType::Float => self.builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs),
                    _ => return Err(GlslError::new(ErrorCode::E0400, format!("nonEqual not supported for {:?}", operand_ty))),
                };
                let one = self.builder.ins().iconst(types::I8, 1);
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().select(cmp_result, one, zero)
            },
            LT => {
                let cmp_result = match operand_ty {
                    GlslType::Int => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
                    GlslType::Float => self.builder.ins().fcmp(FloatCC::LessThan, lhs, rhs),
                    _ => return Err(GlslError::new(ErrorCode::E0400, format!("LT not supported for {:?}", operand_ty))),
                };
                let one = self.builder.ins().iconst(types::I8, 1);
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().select(cmp_result, one, zero)
            },
            GT => {
                let cmp_result = match operand_ty {
                    GlslType::Int => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                    GlslType::Float => self.builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs),
                    _ => return Err(GlslError::new(ErrorCode::E0400, format!("GT not supported for {:?}", operand_ty))),
                };
                let one = self.builder.ins().iconst(types::I8, 1);
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().select(cmp_result, one, zero)
            },
            LTE => {
                let cmp_result = match operand_ty {
                    GlslType::Int => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                    GlslType::Float => self.builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs),
                    _ => return Err(GlslError::new(ErrorCode::E0400, format!("LTE not supported for {:?}", operand_ty))),
                };
                let one = self.builder.ins().iconst(types::I8, 1);
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().select(cmp_result, one, zero)
            },
            GTE => {
                let cmp_result = match operand_ty {
                    GlslType::Int => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
                    GlslType::Float => self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs),
                    _ => return Err(GlslError::new(ErrorCode::E0400, format!("GTE not supported for {:?}", operand_ty))),
                };
                let one = self.builder.ins().iconst(types::I8, 1);
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().select(cmp_result, one, zero)
            },

            // Logical operators (bool only, already validated by type_check)
            And => {
                // Logical AND: both operands must be bool (I8)
                // Result: (lhs != 0) && (rhs != 0) ? 1 : 0
                let zero = self.builder.ins().iconst(types::I8, 0);
                let one = self.builder.ins().iconst(types::I8, 1);
                let lhs_nonzero = self.builder.ins().icmp(IntCC::NotEqual, lhs, zero);
                let rhs_nonzero = self.builder.ins().icmp(IntCC::NotEqual, rhs, zero);
                // Result is 1 if both are non-zero, 0 otherwise
                let rhs_result = self.builder.ins().select(rhs_nonzero, one, zero);
                self.builder.ins().select(lhs_nonzero, rhs_result, zero)
            }
            Or | Xor => {
                return Err(GlslError::new(ErrorCode::E0400, format!("logical operator {:?} not yet implemented", op)));
            }

            _ => return Err(GlslError::new(ErrorCode::E0400, format!("binary operator not supported yet: {:?}", op))),
        };

        Ok(val)
    }

    /// Translate vector binary operation (component-wise or scalar-vector)
    fn translate_vector_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs_vals: Vec<Value>,
        lhs_ty: &GlslType,
        rhs_vals: Vec<Value>,
        rhs_ty: &GlslType,
        span: Option<glsl::syntax::SourceSpan>,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        use glsl::syntax::BinaryOp::*;

        // Validate operation is allowed on vectors
        match op {
            Add | Sub | Mult | Div => {}, // allowed
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("operator {:?} not supported on vectors yet", op))),
        }

        // Determine result type and operation mode
        enum VectorOpMode {
            ComponentWise,    // vec + vec
            VectorScalar,     // vec + scalar
            ScalarVector,     // scalar + vec
        }

        let (result_ty, mode) = if lhs_ty.is_vector() && rhs_ty.is_vector() {
            // vec + vec: component-wise, types must match
            if lhs_ty != rhs_ty {
                let mut error = GlslError::new(ErrorCode::E0106, format!(
                    "vector operation requires matching types, got {:?} and {:?}",
                    lhs_ty, rhs_ty
                ));
                if let Some(s) = span {
                    error = error.with_location(source_span_to_location(&s));
                }
                return Err(error);
            }
            (lhs_ty.clone(), VectorOpMode::ComponentWise)
        } else if lhs_ty.is_vector() && rhs_ty.is_scalar() {
            // vec + scalar: scalar applied to each component
            let vec_base = lhs_ty.vector_base_type().unwrap();
            // Check if scalar can be used with vector base type
            if !rhs_ty.is_numeric() || !vec_base.is_numeric() {
                return Err(GlslError::new(ErrorCode::E0106, format!(
                    "cannot use {:?} with {:?}",
                    rhs_ty, lhs_ty
                )));
            }
            (lhs_ty.clone(), VectorOpMode::VectorScalar)
        } else if lhs_ty.is_scalar() && rhs_ty.is_vector() {
            // scalar + vec: scalar applied to each component
            let vec_base = rhs_ty.vector_base_type().unwrap();
            if !lhs_ty.is_numeric() || !vec_base.is_numeric() {
                return Err(GlslError::new(ErrorCode::E0400, format!(
                    "Cannot use {:?} with {:?}",
                    lhs_ty, rhs_ty
                )));
            }
            (rhs_ty.clone(), VectorOpMode::ScalarVector)
        } else {
            unreachable!("translate_vector_binary_op called with non-vector types");
        };

        let base_ty = result_ty.vector_base_type().unwrap();
        let component_count = result_ty.component_count().unwrap();

        let mut result_vals = Vec::new();
        
        match mode {
            VectorOpMode::ComponentWise => {
                // vec3(a,b,c) + vec3(d,e,f) = vec3(a+d, b+e, c+f)
                for i in 0..component_count {
                    let lhs_comp = lhs_vals[i];
                    let rhs_comp = rhs_vals[i];
                    let result_comp = self.translate_scalar_binary_op(op, lhs_comp, rhs_comp, &base_ty)?;
                    result_vals.push(result_comp);
                }
            }
            VectorOpMode::VectorScalar => {
                // vec3(a,b,c) * s = vec3(a*s, b*s, c*s)
                let scalar = rhs_vals[0];
                // Coerce scalar to vector base type if needed
                let scalar = self.coerce_to_type(scalar, rhs_ty, &base_ty)?;
                for &comp in &lhs_vals {
                    let result_comp = self.translate_scalar_binary_op(op, comp, scalar, &base_ty)?;
                    result_vals.push(result_comp);
                }
            }
            VectorOpMode::ScalarVector => {
                // s * vec3(a,b,c) = vec3(s*a, s*b, s*c)
                let scalar = lhs_vals[0];
                // Coerce scalar to vector base type if needed
                let scalar = self.coerce_to_type(scalar, lhs_ty, &base_ty)?;
                for &comp in &rhs_vals {
                    let result_comp = self.translate_scalar_binary_op(op, scalar, comp, &base_ty)?;
                    result_vals.push(result_comp);
                }
            }
        }

        Ok((result_vals, result_ty))
    }

    /// Translate matrix binary operation
    /// Implements GLSL spec: operators.adoc:1019-1098
    fn translate_matrix_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs_vals: Vec<Value>,
        lhs_ty: &GlslType,
        rhs_vals: Vec<Value>,
        rhs_ty: &GlslType,
        span: glsl::syntax::SourceSpan,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        use glsl::syntax::BinaryOp::*;

        // Infer result type (validates operation is allowed)
        let result_ty = infer_binary_result_type(op, lhs_ty, rhs_ty, span.clone())?;

        match op {
            // Matrix + Matrix: component-wise addition
            Add => {
                if lhs_ty != rhs_ty {
                    return Err(GlslError::new(ErrorCode::E0106, "matrix addition requires matching types")
                        .with_location(source_span_to_location(&span)));
                }
                let mut result_vals = Vec::new();
                for (lhs_val, rhs_val) in lhs_vals.iter().zip(rhs_vals.iter()) {
                    result_vals.push(self.builder.ins().fadd(*lhs_val, *rhs_val));
                }
                Ok((result_vals, result_ty))
            }

            // Matrix - Matrix: component-wise subtraction
            Sub => {
                if lhs_ty != rhs_ty {
                    return Err(GlslError::new(ErrorCode::E0106, "matrix subtraction requires matching types")
                        .with_location(source_span_to_location(&span)));
                }
                let mut result_vals = Vec::new();
                for (lhs_val, rhs_val) in lhs_vals.iter().zip(rhs_vals.iter()) {
                    result_vals.push(self.builder.ins().fsub(*lhs_val, *rhs_val));
                }
                Ok((result_vals, result_ty))
            }

            // Matrix multiplication
            Mult => {
                // Matrix × Scalar: component-wise multiplication
                if lhs_ty.is_matrix() && rhs_ty.is_scalar() {
                    let scalar = rhs_vals[0];
                    let scalar_float = self.coerce_to_type(scalar, rhs_ty, &GlslType::Float)?;
                    let mut result_vals = Vec::new();
                    for &lhs_val in &lhs_vals {
                        result_vals.push(self.builder.ins().fmul(lhs_val, scalar_float));
                    }
                    return Ok((result_vals, result_ty));
                }

                // Scalar × Matrix: component-wise multiplication
                if lhs_ty.is_scalar() && rhs_ty.is_matrix() {
                    let scalar = lhs_vals[0];
                    let scalar_float = self.coerce_to_type(scalar, lhs_ty, &GlslType::Float)?;
                    let mut result_vals = Vec::new();
                    for &rhs_val in &rhs_vals {
                        result_vals.push(self.builder.ins().fmul(scalar_float, rhs_val));
                    }
                    return Ok((result_vals, result_ty));
                }

                // Matrix × Vector: linear algebra multiplication
                if lhs_ty.is_matrix() && rhs_ty.is_vector() {
                    let (rows, cols) = lhs_ty.matrix_dims().unwrap();
                    let vec_size = rhs_ty.component_count().unwrap();
                    
                    if cols != vec_size {
                        return Err(GlslError::new(ErrorCode::E0106, 
                            format!("matrix × vector dimension mismatch: {}×{} matrix requires {}-component vector", rows, cols, cols))
                            .with_location(source_span_to_location(&span)));
                    }

                    // Result is a vector with rows components
                    // For each row i: result[i] = dot(row i of matrix, vector)
                    let mut result_vals = Vec::new();
                    for row in 0..rows {
                        let mut sum = self.builder.ins().fmul(
                            lhs_vals[0 * rows + row], // First element of row
                            rhs_vals[0]
                        );
                        for col in 1..cols {
                            let product = self.builder.ins().fmul(
                                lhs_vals[col * rows + row], // Element at (row, col)
                                rhs_vals[col]
                            );
                            sum = self.builder.ins().fadd(sum, product);
                        }
                        result_vals.push(sum);
                    }
                    return Ok((result_vals, result_ty));
                }

                // Vector × Matrix: linear algebra multiplication
                if lhs_ty.is_vector() && rhs_ty.is_matrix() {
                    let vec_size = lhs_ty.component_count().unwrap();
                    let (rows, cols) = rhs_ty.matrix_dims().unwrap();
                    
                    if vec_size != rows {
                        return Err(GlslError::new(ErrorCode::E0106,
                            format!("vector × matrix dimension mismatch: {}-component vector requires {}×{} matrix", vec_size, rows, cols))
                            .with_location(source_span_to_location(&span)));
                    }

                    // Result is a vector with cols components
                    // For each column j: result[j] = dot(vector, column j of matrix)
                    let mut result_vals = Vec::new();
                    for col in 0..cols {
                        let mut sum = self.builder.ins().fmul(
                            lhs_vals[0],
                            rhs_vals[col * rows + 0] // First element of column
                        );
                        for row in 1..rows {
                            let product = self.builder.ins().fmul(
                                lhs_vals[row],
                                rhs_vals[col * rows + row] // Element at (row, col)
                            );
                            sum = self.builder.ins().fadd(sum, product);
                        }
                        result_vals.push(sum);
                    }
                    return Ok((result_vals, result_ty));
                }

                // Matrix × Matrix: linear algebra multiplication
                if lhs_ty.is_matrix() && rhs_ty.is_matrix() {
                    let (lhs_rows, lhs_cols) = lhs_ty.matrix_dims().unwrap();
                    let (rhs_rows, rhs_cols) = rhs_ty.matrix_dims().unwrap();
                    
                    if lhs_cols != rhs_rows {
                        return Err(GlslError::new(ErrorCode::E0106,
                            format!("matrix × matrix dimension mismatch: {}×{} × {}×{} requires {} == {}", 
                                lhs_rows, lhs_cols, rhs_rows, rhs_cols, lhs_cols, rhs_rows))
                            .with_location(source_span_to_location(&span)));
                    }

                    // Result is lhs_rows × rhs_cols matrix
                    // For each (i, j): result[i][j] = dot(row i of lhs, column j of rhs)
                    let mut result_vals = Vec::new();
                    for i in 0..lhs_rows {
                        for j in 0..rhs_cols {
                            // Dot product of row i of lhs with column j of rhs
                            let mut sum = self.builder.ins().fmul(
                                lhs_vals[0 * lhs_rows + i], // Element at (i, 0) of lhs
                                rhs_vals[j * rhs_rows + 0]  // Element at (0, j) of rhs
                            );
                            for k in 1..lhs_cols {
                                let product = self.builder.ins().fmul(
                                    lhs_vals[k * lhs_rows + i], // Element at (i, k) of lhs
                                    rhs_vals[j * rhs_rows + k]  // Element at (k, j) of rhs
                                );
                                sum = self.builder.ins().fadd(sum, product);
                            }
                            result_vals.push(sum);
                        }
                    }
                    return Ok((result_vals, result_ty));
                }

                Err(GlslError::new(ErrorCode::E0106, "matrix multiplication requires matrix and scalar/vector/matrix operands")
                    .with_location(source_span_to_location(&span)))
            }

            // Matrix / Scalar: component-wise division
            Div => {
                if lhs_ty.is_matrix() && rhs_ty.is_scalar() {
                    let scalar = rhs_vals[0];
                    let scalar_float = self.coerce_to_type(scalar, rhs_ty, &GlslType::Float)?;
                    let mut result_vals = Vec::new();
                    for &lhs_val in &lhs_vals {
                        result_vals.push(self.builder.ins().fdiv(lhs_val, scalar_float));
                    }
                    return Ok((result_vals, result_ty));
                }
                Err(GlslError::new(ErrorCode::E0106, "matrix division only supports matrix / scalar")
                    .with_location(source_span_to_location(&span)))
            }

            _ => Err(GlslError::new(ErrorCode::E0106, format!("operator {:?} not supported for matrices", op))
                .with_location(source_span_to_location(&span))),
        }
    }

    fn translate_unary_op(
        &mut self,
        op: &glsl::syntax::UnaryOp,
        val: Value,
        operand_ty: &GlslType,
    ) -> Result<Value, GlslError> {
        use glsl::syntax::UnaryOp::*;

        let result = match op {
            Minus => match operand_ty {
                GlslType::Int => self.builder.ins().ineg(val),
                GlslType::Float => self.builder.ins().fneg(val),
                _ => return Err(GlslError::new(ErrorCode::E0400, format!("unary minus not supported for {:?}", operand_ty))),
            },
            Not => {
                if operand_ty != &GlslType::Bool {
                    return Err(GlslError::new(ErrorCode::E0107, format!("logical NOT requires bool, got {:?}", operand_ty)));
                }
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().icmp(IntCC::Equal, val, zero)
            }
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("unary operator not supported yet: {:?}", op))),
        };

        Ok(result)
    }

    /// Coerce a value from one type to another (implements GLSL implicit conversions)
    pub fn coerce_to_type(
        &mut self,
        val: Value,
        from_ty: &GlslType,
        to_ty: &GlslType,
    ) -> Result<Value, GlslError> {
        self.coerce_to_type_with_location(val, from_ty, to_ty, None)
    }

    pub fn coerce_to_type_with_location(
        &mut self,
        val: Value,
        from_ty: &GlslType,
        to_ty: &GlslType,
        span: Option<glsl::syntax::SourceSpan>,
    ) -> Result<Value, GlslError> {
        use crate::error::source_span_to_location;
        
        if from_ty == to_ty {
            return Ok(val);
        }

        match (from_ty, to_ty) {
            (GlslType::Int, GlslType::Float) => {
                // int → float: fcvt_from_sint
                Ok(self.builder.ins().fcvt_from_sint(types::F32, val))
            }
            _ => {
                // For return type mismatches, match the verification error message format
                let error_msg = if span.is_some() {
                    format!("code generation failed: Compilation error: Verifier errors")
                } else {
                    format!("code generation failed: cannot implicitly convert {:?} to {:?}", from_ty, to_ty)
                };
                let mut error = GlslError::new(ErrorCode::E0400, error_msg);
                if let Some(s) = span {
                    error = error.with_location(source_span_to_location(&s));
                }
                Err(error)
            },
        }
    }

    fn translate_assignment_typed(
        &mut self,
        lhs: &Expr,
        op: &glsl::syntax::AssignmentOp,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        // Only simple assignment (=) for now
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return Err(GlslError::new(ErrorCode::E0400, "only simple assignment (=) supported"));
        }

        // Check if lhs is component access
        if let Expr::Dot(vec_expr, field, _span) = lhs {
            return self.translate_component_assignment(vec_expr, field, rhs);
        }

        // Get variable name from lhs
        let var_name = match lhs {
            Expr::Variable(ident, _span) => &ident.name,
            _ => {
                let span = extract_span_from_expr(lhs);
                let error = GlslError::new(ErrorCode::E0115, "assignment lhs must be variable")
                    .with_location(source_span_to_location(&span));
                return Err(self.add_span_to_error(error, &span));
            }
        };

        let vars = self
            .lookup_variables(var_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable `{}` not found", var_name)))?
            .to_vec(); // Clone to avoid borrow issues

        let lhs_ty = self
            .lookup_variable_type(var_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable type not found for `{}`", var_name)))?
            .clone();

        // Translate RHS
        let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;

        // Validate assignment (check implicit conversion is allowed)
        let rhs_span = extract_span_from_expr(rhs);
        match check_assignment(&lhs_ty, &rhs_ty) {
            Ok(()) => {}
            Err(mut error) => {
                if error.location.is_none() {
                    error = error.with_location(source_span_to_location(&rhs_span));
                }
                return Err(self.add_span_to_error(error, &rhs_span));
            }
        }

        // Check component counts match
        if vars.len() != rhs_vals.len() {
            return Err(GlslError::new(ErrorCode::E0400, format!(
                "component count mismatch in assignment: {} vs {}",
                vars.len(), rhs_vals.len()
            )));
        }

        // Coerce and assign each component
        let rhs_base = if rhs_ty.is_vector() {
            rhs_ty.vector_base_type().unwrap()
        } else {
            rhs_ty.clone()
        };
        let lhs_base = if lhs_ty.is_vector() {
            lhs_ty.vector_base_type().unwrap()
        } else {
            lhs_ty.clone()
        };

        let mut coerced_vals = Vec::new();
        for (var, val) in vars.iter().zip(&rhs_vals) {
            let coerced = self.coerce_to_type(*val, &rhs_base, &lhs_base)?;
            self.builder.def_var(*var, coerced);
            coerced_vals.push(coerced);
        }

        // Assignment expression has same type as LHS
        Ok((coerced_vals, lhs_ty))
    }

    fn translate_component_assignment(
        &mut self,
        vec_expr: &Expr,
        field: &glsl::syntax::Identifier,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        // Get variable name
        let var_name = match vec_expr {
            Expr::Variable(ident, _span) => &ident.name,
            _ => return Err(GlslError::new(ErrorCode::E0400, "component assignment only supported on variables")),
        };

        let vars = self.lookup_variables(var_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable `{}` not found", var_name)))?
            .to_vec(); // Clone to avoid borrow issues
        let vec_ty = self.lookup_variable_type(var_name)
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, format!("variable type not found for `{}`", var_name)))?
            .clone();

        if !vec_ty.is_vector() {
            return Err(GlslError::new(ErrorCode::E0112, format!("component access on non-vector variable: {}", var_name)));
        }

        // Parse swizzle (supports multi-component assignment)
        // Extract span from field identifier for error reporting
        let field_span = extract_span_from_identifier(field);
        let indices = Self::parse_vector_swizzle(&field.name, &vec_ty, Some(field_span))?;
        let base_ty = vec_ty.vector_base_type().unwrap();

        // Check for duplicates (illegal in assignment LHS)
        if Self::has_duplicates(&indices) {
            let span = extract_span_from_identifier(field);
            let error = GlslError::new(ErrorCode::E0113, format!("swizzle `{}` contains duplicate components (illegal in assignment)", field.name))
                .with_location(source_span_to_location(&span));
            return Err(self.add_span_to_error(error, &span));
        }

        // Translate RHS
        let rhs_span = extract_span_from_expr(rhs);
        let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;
        
        // Validate sizes match
        if rhs_vals.len() != indices.len() {
            let error = GlslError::new(ErrorCode::E0400, format!(
                "swizzle assignment size mismatch: {} components on LHS, {} on RHS",
                indices.len(), rhs_vals.len()
            ))
            .with_location(source_span_to_location(&rhs_span));
            return Err(self.add_span_to_error(error, &rhs_span));
        }

        // Type check base types
        let rhs_base = if rhs_ty.is_vector() {
            rhs_ty.vector_base_type().unwrap()
        } else {
            rhs_ty.clone()
        };
        check_assignment(&base_ty, &rhs_base)?;

        // Assign each component
        for (i, &idx) in indices.iter().enumerate() {
            let rhs_val = self.coerce_to_type(rhs_vals[i], &rhs_base, &base_ty)?;
            self.builder.def_var(vars[idx], rhs_val);
        }

        // Return all current values (read other components)
        let mut result_vals = Vec::new();
        for &var in &vars {
            result_vals.push(self.builder.use_var(var));
        }

        // Component assignment returns the whole vector
        Ok((result_vals, vec_ty))
    }

    fn translate_vector_constructor(
        &mut self,
        type_name: &str,
        args: &[Expr],
        span: glsl::syntax::SourceSpan,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        use crate::error::source_span_to_location;
        
        // Translate all arguments
        let mut arg_vals: Vec<Vec<Value>> = Vec::new();
        let mut arg_types: Vec<GlslType> = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            arg_vals.push(vals);
            arg_types.push(ty);
        }

        // Type check constructor
        let result_type = match check_vector_constructor_with_span(type_name, &arg_types, Some(span.clone())) {
            Ok(ty) => ty,
            Err(mut error) => {
                // Ensure error has location and span_text
                if error.location.is_none() {
                    error = error.with_location(source_span_to_location(&span));
                }
                return Err(self.add_span_to_error(error, &span));
            }
        };
        let base_type = result_type.vector_base_type().unwrap();
        let component_count = result_type.component_count().unwrap();

        // Generate component values
        let mut components = Vec::new();

        // Case 1: Single scalar broadcast
        if arg_types.len() == 1 && arg_types[0].is_scalar() {
            let scalar = arg_vals[0][0];
            let coerced = self.coerce_to_type(scalar, &arg_types[0], &base_type)?;
            for _ in 0..component_count {
                components.push(coerced);
            }
        }
        // Case 2: Single vector conversion
        else if arg_types.len() == 1 && arg_types[0].is_vector() {
            let src_base = arg_types[0].vector_base_type().unwrap();
            for val in &arg_vals[0] {
                components.push(self.coerce_to_type(*val, &src_base, &base_type)?);
            }
        }
        // Case 3: Concatenation
        else {
            for (vals, ty) in arg_vals.iter().zip(&arg_types) {
                let arg_base = if ty.is_vector() {
                    ty.vector_base_type().unwrap()
                } else {
                    ty.clone()
                };
                
                for &val in vals {
                    components.push(self.coerce_to_type(val, &arg_base, &base_type)?);
                }
            }
        }

        Ok((components, result_type))
    }

    /// Translate matrix constructor
    /// Implements GLSL spec: variables.adoc:72-97
    fn translate_matrix_constructor(
        &mut self,
        type_name: &str,
        args: &[Expr],
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        // Translate all arguments
        let mut arg_vals: Vec<Vec<Value>> = Vec::new();
        let mut arg_types: Vec<GlslType> = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            arg_vals.push(vals);
            arg_types.push(ty);
        }

        // Type check constructor
        let result_type = check_matrix_constructor(type_name, &arg_types)?;
        let (rows, cols) = result_type.matrix_dims().unwrap();
        let element_count = rows * cols;

        // Allocate temporary variables for matrix elements
        let mut matrix_vars = Vec::new();
        for _ in 0..element_count {
            let var = self.builder.declare_var(cranelift_codegen::ir::types::F32);
            matrix_vars.push(var);
        }

        // Case 1: Single scalar - identity matrix (diagonal = scalar, rest = 0.0)
        if arg_types.len() == 1 && arg_types[0].is_scalar() {
            let scalar = arg_vals[0][0];
            let scalar_float = self.coerce_to_type(scalar, &arg_types[0], &GlslType::Float)?;
            let zero = self.builder.ins().f32const(0.0);
            
            for row in 0..rows {
                for col in 0..cols {
                    let value = if row == col { scalar_float } else { zero };
                    self.builder.def_var(matrix_vars[col * rows + row], value);
                }
            }
        }
        // Case 2: Column vectors - one vector per column
        else if arg_types.len() == cols {
            for (col, (vals, ty)) in arg_vals.iter().zip(&arg_types).enumerate() {
                let vec_base = ty.vector_base_type().unwrap();
                for (row, &val) in vals.iter().enumerate() {
                    let float_val = self.coerce_to_type(val, &vec_base, &GlslType::Float)?;
                    self.builder.def_var(matrix_vars[col * rows + row], float_val);
                }
            }
        }
        // Case 3: Mixed scalars - column-major order
        else {
            let mut scalar_index = 0;
            for col in 0..cols {
                for row in 0..rows {
                    let scalar = arg_vals[scalar_index][0];
                    let scalar_ty = &arg_types[scalar_index];
                    let float_val = self.coerce_to_type(scalar, scalar_ty, &GlslType::Float)?;
                    self.builder.def_var(matrix_vars[col * rows + row], float_val);
                    scalar_index += 1;
                }
            }
        }

        // Return all matrix element values
        let mut result_vals = Vec::new();
        for &var in &matrix_vars {
            result_vals.push(self.builder.use_var(var));
        }

        Ok((result_vals, result_type))
    }

    /// Translate built-in function call
    fn translate_builtin_call_expr(
        &mut self,
        name: &str,
        args: &[glsl::syntax::Expr],
        call_span: glsl::syntax::SourceSpan,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {

        // Translate all arguments
        let mut translated_args = Vec::new();
        let mut arg_types = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            translated_args.push((vals, ty.clone()));
            arg_types.push(ty);
        }
        
        // Validate builtin call before codegen
        match crate::semantic::builtins::check_builtin_call(name, &arg_types) {
            Ok(_return_type) => {
                // Validation passed, proceed with codegen
            }
            Err(err_msg) => {
                // Convert validation error to GlslError
                let error = GlslError::new(
                    crate::error::ErrorCode::E0114,
                    err_msg,
                )
                .with_location(source_span_to_location(&call_span));
                return Err(self.add_span_to_error(error, &call_span));
            }
        }
        
        // Delegate to built-in implementation and add span to any errors
        match self.translate_builtin_call(name, translated_args) {
            Ok(result) => Ok(result),
            Err(mut error) => {
                // Add location and span_text if not already present
                if error.location.is_none() {
                    error = error.with_location(source_span_to_location(&call_span));
                }
                Err(self.add_span_to_error(error, &call_span))
            }
        }
    }

    /// Translate user-defined function call
    fn translate_user_function_call(
        &mut self,
        name: &str,
        args: &[glsl::syntax::Expr],
        call_span: glsl::syntax::SourceSpan,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        // Translate arguments and collect types first (requires mutable borrow)
        let mut arg_vals_flat = Vec::new();
        let mut arg_types = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            arg_vals_flat.extend(vals);
            arg_types.push(ty);
        }

        // Now get function IDs and registry (immutable borrow)
        let func_ids = self.function_ids.as_ref()
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "function IDs not set (internal error)"))?;
        let func_registry = self.function_registry
            .ok_or_else(|| GlslError::new(ErrorCode::E0400, "function registry not set (internal error)"))?;

        // Lookup function signature - this will return E0114 if no match
        let func_id = func_ids.get(name)
            .ok_or_else(|| {
                let error = GlslError::undefined_function(name);
                if error.location.is_none() {
                    error.with_location(crate::error::source_span_to_location(&call_span))
                } else {
                    error
                }
            })?;
        let func_sig = match func_registry.lookup_function(name, &arg_types) {
            Ok(sig) => sig,
            Err(mut error) => {
                // Ensure error has location
                if error.location.is_none() {
                    error = error.with_location(crate::error::source_span_to_location(&call_span));
                }
                return Err(self.add_span_to_error(error, &call_span));
            }
        };

        // Validate that all arguments can be coerced BEFORE declaring the function
        // This prevents function context pollution if coercion fails
        for (param, arg_ty) in func_sig.parameters.iter().zip(&arg_types) {
            let arg_base = if arg_ty.is_vector() {
                arg_ty.vector_base_type().unwrap()
            } else {
                arg_ty.clone()
            };
            let param_base = if param.ty.is_vector() {
                param.ty.vector_base_type().unwrap()
            } else {
                param.ty.clone()
            };
            
            // Check if coercion is possible
            if arg_base != param_base && !crate::semantic::type_check::can_implicitly_convert(&arg_base, &param_base) {
                // Calculate expected parameter count for error message
                let expected_count: usize = func_sig.parameters.iter().map(|p| {
                    if p.ty.is_vector() {
                        p.ty.component_count().unwrap()
                    } else {
                        1
                    }
                }).sum();
                let error = GlslError::new(
                    ErrorCode::E0400,
                    format!("function parameter mismatch: expected {} block parameters, got 0", expected_count)
                )
                .with_location(crate::error::source_span_to_location(&call_span))
                .with_note(format!("function `{}` expects parameter of type `{:?}`, got `{:?}`", name, param.ty, arg_ty));
                return Err(self.add_span_to_error(error, &call_span));
            }
        }

        // Import the function into the current function to get a FuncRef
        let func_ref = self.module.declare_func_in_func(*func_id, self.builder.func);

        // Type check and prepare arguments (with implicit conversions)
        let mut call_args = Vec::new();
        let mut arg_val_idx = 0;
        
        for (param, arg_ty) in func_sig.parameters.iter().zip(&arg_types) {
            let arg_base = if arg_ty.is_vector() {
                arg_ty.vector_base_type().unwrap()
            } else {
                arg_ty.clone()
            };
            let param_base = if param.ty.is_vector() {
                param.ty.vector_base_type().unwrap()
            } else {
                param.ty.clone()
            };
            
            let component_count = if arg_ty.is_vector() {
                arg_ty.component_count().unwrap()
            } else {
                1
            };
            
            for _ in 0..component_count {
                let arg_val = arg_vals_flat[arg_val_idx];
                let converted = self.coerce_to_type_with_location(arg_val, &arg_base, &param_base, Some(call_span.clone()))?;
                call_args.push(converted);
                arg_val_idx += 1;
            }
        }

        // Make the function call
        let call_inst = self.builder.ins().call(func_ref, &call_args);
        
        // Get return values
        let return_vals = self.builder.inst_results(call_inst).to_vec();
        
        // Package return value(s)
        if func_sig.return_type == GlslType::Void {
            Ok((vec![], GlslType::Void))
        } else if func_sig.return_type.is_vector() {
            let count = func_sig.return_type.component_count().unwrap();
            Ok((return_vals[0..count].to_vec(), func_sig.return_type.clone()))
        } else {
            Ok((vec![return_vals[0]], func_sig.return_type.clone()))
        }
    }

    /// Parse vector component swizzle and return indices
    /// Supports xyzw, rgba, stpq naming sets
    /// Can parse multiple components: "xy", "rgba", "zyx", "xxxx", etc.
    fn parse_vector_swizzle(name: &str, vec_ty: &GlslType, span: Option<glsl::syntax::SourceSpan>) -> Result<Vec<usize>, GlslError> {
        if name.is_empty() {
            return Err(GlslError::new(ErrorCode::E0113, "empty swizzle"));
        }
        
        if name.len() > 4 {
            return Err(GlslError::new(ErrorCode::E0113, format!("swizzle can have at most 4 components, got {}", name.len())));
        }
        
        let component_count = vec_ty.component_count()
            .ok_or_else(|| GlslError::new(ErrorCode::E0112, format!("{:?} is not a vector type", vec_ty)))?;
        
        // Determine which naming set is used and validate consistency
        let naming_set = Self::determine_naming_set(name)?;
        
        // Parse each character
        let mut indices = Vec::new();
        for ch in name.chars() {
            let idx = Self::parse_single_component(ch, naming_set)?;
            
            // Validate index is within bounds
            if idx >= component_count {
                let mut error = GlslError::new(ErrorCode::E0111, format!(
                    "component '{}' not valid for {:?} (has only {} components)",
                    ch, vec_ty, component_count
                ));
                if let Some(s) = span {
                    error = error.with_location(source_span_to_location(&s));
                }
                return Err(error);
            }
            
            indices.push(idx);
        }
        
        Ok(indices)
    }

    /// Determine which naming set is used in a swizzle and validate consistency
    fn determine_naming_set(swizzle: &str) -> Result<NamingSet, GlslError> {
        let mut xyzw_count = 0;
        let mut rgba_count = 0;
        let mut stpq_count = 0;
        
        for ch in swizzle.chars() {
            match ch {
                'x' | 'y' | 'z' | 'w' => xyzw_count += 1,
                'r' | 'g' | 'b' | 'a' => rgba_count += 1,
                's' | 't' | 'p' | 'q' => stpq_count += 1,
                _ => return Err(GlslError::new(ErrorCode::E0113, format!("invalid swizzle character: '{}'", ch))),
            }
        }
        
        let sets_used = (xyzw_count > 0) as u32 + (rgba_count > 0) as u32 + (stpq_count > 0) as u32;
        
        if sets_used > 1 {
            return Err(GlslError::new(ErrorCode::E0113, format!("swizzle '{}' mixes component naming sets (xyzw/rgba/stpq)", swizzle)));
        }
        
        if xyzw_count > 0 { 
            Ok(NamingSet::XYZW) 
        } else if rgba_count > 0 { 
            Ok(NamingSet::RGBA) 
        } else { 
            Ok(NamingSet::STPQ) 
        }
    }

    /// Parse a single component character given a naming set
    fn parse_single_component(ch: char, naming_set: NamingSet) -> Result<usize, GlslError> {
        match naming_set {
            NamingSet::XYZW => match ch {
                'x' => Ok(0),
                'y' => Ok(1),
                'z' => Ok(2),
                'w' => Ok(3),
                _ => Err(GlslError::new(ErrorCode::E0113, format!("invalid component '{}' for xyzw naming set", ch))),
            },
            NamingSet::RGBA => match ch {
                'r' => Ok(0),
                'g' => Ok(1),
                'b' => Ok(2),
                'a' => Ok(3),
                _ => Err(GlslError::new(ErrorCode::E0113, format!("invalid component '{}' for rgba naming set", ch))),
            },
            NamingSet::STPQ => match ch {
                's' => Ok(0),
                't' => Ok(1),
                'p' => Ok(2),
                'q' => Ok(3),
                _ => Err(GlslError::new(ErrorCode::E0113, format!("invalid component '{}' for stpq naming set", ch))),
            },
        }
    }

    /// Check if a slice of indices contains duplicates
    fn has_duplicates(indices: &[usize]) -> bool {
        let mut seen = HashSet::new();
        for &idx in indices {
            if !seen.insert(idx) {
                return true;
            }
        }
        false
    }
}

