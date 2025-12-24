//! Expression code generation
//!
//! This module handles translation of GLSL expressions to Cranelift IR.
//! Each expression type has its own module:
//! - `literal`: Integer, float, and boolean literals
//! - `variable`: Variable references
//! - `binary`: Binary operations (+, -, *, /, ==, !=, etc.)
//! - `unary`: Unary operations (-, !)
//! - `function`: Function calls (built-in and user-defined)
//! - `constructor`: Type constructors (vec3(), mat2(), etc.)
//! - `vector`: Vector operations
//! - `matrix`: Matrix operations
//! - `component`: Component access and swizzling
//! - `coercion`: Type coercion and implicit conversions

pub mod literal;
pub mod variable;
pub mod binary;
pub mod unary;
pub mod function;
pub mod constructor;
pub mod vector;
pub mod matrix;
pub mod component;
pub mod coercion;
pub mod incdec;

use crate::codegen::context::CodegenContext;
use crate::codegen::rvalue::RValue;
use crate::semantic::types::Type as GlslType;
use crate::error::{ErrorCode, GlslError, source_span_to_location};
use glsl::syntax::Expr;
use cranelift_codegen::ir::Value;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

impl<'a> CodegenContext<'a> {
    /// Emit code to compute an RValue (right-hand value)
    ///
    /// This is the new primary entry point for expression evaluation,
    /// following Clang's pattern of separating LValue (locations) from RValue (values).
    pub fn emit_rvalue(&mut self, expr: &Expr) -> Result<RValue, GlslError> {
        // Ensure we're in a block before evaluating expressions
        self.ensure_block()?;
        
        match expr {
            Expr::IntConst(..) | Expr::FloatConst(..) | Expr::BoolConst(..) => {
                literal::emit_literal_rvalue(self, expr)
            }
            Expr::Variable(..) => {
                // Read variable as RValue: resolve LValue then load it
                let lvalue = self.emit_lvalue(expr)?;
                self.load_lvalue(lvalue)
            }
            Expr::Binary(..) => {
                binary::emit_binary_rvalue(self, expr)
            }
            Expr::Unary(op, operand, span) => {
                // Handle pre-increment/decrement specially
                use glsl::syntax::UnaryOp::*;
                match op {
                    Inc => {
                        // Pre-increment returns RValue
                        let (vals, ty) = incdec::translate_preinc(self, operand, span.clone())?;
                        Ok(RValue::from_aggregate(vals, ty))
                    }
                    Dec => {
                        // Pre-decrement returns RValue
                        let (vals, ty) = incdec::translate_predec(self, operand, span.clone())?;
                        Ok(RValue::from_aggregate(vals, ty))
                    }
                    _ => unary::emit_unary_rvalue(self, expr),
                }
            }
            Expr::FunCall(..) => {
                function::emit_function_call_rvalue(self, expr)
            }
            Expr::Dot(..) => {
                // Component access: resolve as LValue then load
                let lvalue = self.emit_lvalue(expr)?;
                self.load_lvalue(lvalue)
            }
            Expr::Bracket(..) => {
                // Matrix indexing: resolve as LValue then load
                let lvalue = self.emit_lvalue(expr)?;
                self.load_lvalue(lvalue)
            }
            Expr::Assignment(lhs, op, rhs, _span) => {
                // Assignment expression: evaluate and return RValue
                let (vals, ty) = self.translate_assignment_typed(lhs, op, rhs)?;
                Ok(RValue::from_aggregate(vals, ty))
            }
            Expr::PostInc(operand, span) => {
                // Post-increment returns RValue (original value)
                let (vals, ty) = incdec::translate_postinc(self, operand, span.clone())?;
                Ok(RValue::from_aggregate(vals, ty))
            }
            Expr::PostDec(operand, span) => {
                // Post-decrement returns RValue (original value)
                let (vals, ty) = incdec::translate_postdec(self, operand, span.clone())?;
                Ok(RValue::from_aggregate(vals, ty))
            }
            _ => Err(GlslError::new(ErrorCode::E0400, format!("expression not supported yet: {:?}", expr))),
        }
    }

    /// Emit code to compute an LValue (left-hand value - modifiable location)
    ///
    /// This resolves an expression to a modifiable location, following Clang's pattern.
    pub fn emit_lvalue(&mut self, expr: &Expr) -> Result<crate::codegen::lvalue::LValue, GlslError> {
        use crate::codegen::lvalue::resolve_lvalue;
        resolve_lvalue(self, expr)
    }

    /// Load an LValue to get its RValue
    ///
    /// This reads the current value(s) from a modifiable location.
    pub fn load_lvalue(&mut self, lvalue: crate::codegen::lvalue::LValue) -> Result<RValue, GlslError> {
        use crate::codegen::lvalue::read_lvalue;
        let (vals, ty) = read_lvalue(self, &lvalue)?;
        Ok(RValue::from_aggregate(vals, ty))
    }

    /// Main entry point for expression translation (legacy - use emit_rvalue instead)
    ///
    /// This method is kept for backwards compatibility during the transition.
    /// New code should use `emit_rvalue` instead.
    pub fn translate_expr_typed(&mut self, expr: &Expr) -> Result<(Vec<Value>, GlslType), GlslError> {
        let rvalue = self.emit_rvalue(expr)?;
        let ty = rvalue.ty().clone();
        Ok((rvalue.into_values(), ty))
    }

    /// Legacy wrapper for compatibility - returns just the first value (for scalars)
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<Value, GlslError> {
        let (vals, _ty) = self.translate_expr_typed(expr)?;
        vals.into_iter().next().ok_or_else(|| GlslError::new(ErrorCode::E0400, "expression produced no values"))
    }

    // Assignment translation (needs to stay here as it's used by stmt.rs)
    fn translate_assignment_typed(
        &mut self,
        lhs: &Expr,
        op: &glsl::syntax::AssignmentOp,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        use crate::codegen::lvalue::write_lvalue;
        use crate::error::extract_span_from_expr;
        use crate::error::extract_span_from_identifier;
        use crate::semantic::type_check::check_assignment;
        use component;

        // Handle compound assignment operators (+=, -=, *=, /=)
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return self.translate_compound_assignment_typed(lhs, op, rhs);
        }

        // Resolve LHS to an LValue
        let lvalue = self.emit_lvalue(lhs)?;
        let lhs_ty = lvalue.ty();

        // Special handling for component assignment with swizzles (check for duplicates)
        if let crate::codegen::lvalue::LValue::Component { indices, .. } = &lvalue {
            // Check for duplicates (illegal in assignment LHS)
            if component::has_duplicates(indices) {
                // Try to extract span from the field identifier
                if let Expr::Dot(_, field, _) = lhs {
                    let span = extract_span_from_identifier(field);
                    let error = GlslError::new(
                        ErrorCode::E0113,
                        format!("swizzle `{}` contains duplicate components (illegal in assignment)", field.name)
                    )
                    .with_location(source_span_to_location(&span));
                    return Err(self.add_span_to_error(error, &span));
                }
            }
        }

        // Translate RHS as RValue
        let rhs_rvalue = self.emit_rvalue(rhs)?;
        let rhs_ty = rhs_rvalue.ty().clone();
        let rhs_vals = rhs_rvalue.into_values();

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
        let expected_count = match &lvalue {
            crate::codegen::lvalue::LValue::Variable { vars, .. } => vars.len(),
            crate::codegen::lvalue::LValue::Component { indices, .. } => indices.len(),
            crate::codegen::lvalue::LValue::MatrixElement { .. } => 1,
            crate::codegen::lvalue::LValue::MatrixColumn { result_ty, .. } => {
                result_ty.component_count().unwrap()
            }
            crate::codegen::lvalue::LValue::VectorElement { .. } => 1,
        };

        if expected_count != rhs_vals.len() {
            return Err(GlslError::new(ErrorCode::E0400, format!(
                "component count mismatch in assignment: {} vs {}",
                expected_count, rhs_vals.len()
            ))
            .with_location(source_span_to_location(&rhs_span)));
        }

        // Coerce and assign each component
        let rhs_base = if rhs_ty.is_vector() {
            rhs_ty.vector_base_type().unwrap()
        } else if rhs_ty.is_matrix() {
            GlslType::Float // Matrix elements are always float
        } else {
            rhs_ty.clone()
        };
        let lhs_base = if lhs_ty.is_vector() {
            lhs_ty.vector_base_type().unwrap()
        } else if lhs_ty.is_matrix() {
            GlslType::Float // Matrix elements are always float
        } else {
            lhs_ty.clone()
        };

        let mut coerced_vals = Vec::new();
        for val in &rhs_vals {
            let coerced = coercion::coerce_to_type(self, *val, &rhs_base, &lhs_base)?;
            coerced_vals.push(coerced);
        }

        // Write coerced values to LValue
        write_lvalue(self, &lvalue, &coerced_vals)?;

        // For component assignment, return all current values (read other components)
        // For other assignments, return the assigned values
        let result_vals = match &lvalue {
            crate::codegen::lvalue::LValue::Component { base_vars, base_ty, .. } => {
                // Component assignment returns the whole vector/matrix
                let mut result_vals = Vec::new();
                for &var in base_vars {
                    result_vals.push(self.builder.use_var(var));
                }
                (result_vals, base_ty.clone())
            }
            _ => {
                // Other assignments return the assigned values
                (coerced_vals, lhs_ty)
            }
        };

        Ok(result_vals)
    }

    /// Handle compound assignment operators (+=, -=, *=, /=)
    fn translate_compound_assignment_typed(
        &mut self,
        lhs: &Expr,
        op: &glsl::syntax::AssignmentOp,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), GlslError> {
        use crate::codegen::lvalue::{write_lvalue, read_lvalue};
        use crate::error::extract_span_from_expr;
        use crate::semantic::type_check::check_assignment;
        use matrix;
        use vector;
        use binary;
        use glsl::syntax::BinaryOp;

        // Resolve LHS to an LValue
        let lvalue = self.emit_lvalue(lhs)?;
        let lhs_ty = lvalue.ty();

        // Translate RHS as RValue
        let rhs_rvalue = self.emit_rvalue(rhs)?;
        let rhs_ty = rhs_rvalue.ty().clone();
        let rhs_vals = rhs_rvalue.into_values();
        let rhs_span = extract_span_from_expr(rhs);

        // Validate assignment types
        // For compound assignment, we allow:
        // - Same type operations (matrix + matrix, vector + vector, scalar + scalar)
        // - Scalar operations on matrices/vectors (matrix * scalar, vector * scalar)
        // Only validate direct assignment compatibility for same-type operations
        let is_scalar_op_on_matrix = (lhs_ty.is_matrix() || lhs_ty.is_vector()) && rhs_ty.is_scalar();
        let is_scalar_op_on_vector = lhs_ty.is_vector() && rhs_ty.is_scalar();
        
        if !is_scalar_op_on_matrix && !is_scalar_op_on_vector {
            // For same-type operations, validate assignment compatibility
            match check_assignment(&lhs_ty, &rhs_ty) {
                Ok(()) => {}
                Err(mut error) => {
                    if error.location.is_none() {
                        error = error.with_location(source_span_to_location(&rhs_span));
                    }
                    return Err(self.add_span_to_error(error, &rhs_span));
                }
            }
        }

        // Read current value from LHS
        let (lhs_vals, _) = read_lvalue(self, &lvalue)?;

        // Convert assignment operator to binary operator
        let binary_op = match op {
            glsl::syntax::AssignmentOp::Add => BinaryOp::Add,
            glsl::syntax::AssignmentOp::Sub => BinaryOp::Sub,
            glsl::syntax::AssignmentOp::Mult => BinaryOp::Mult,
            glsl::syntax::AssignmentOp::Div => BinaryOp::Div,
            _ => return Err(GlslError::new(ErrorCode::E0400, format!("unsupported compound assignment operator: {:?}", op))),
        };

        // Perform the compound operation
        let (operation_result_vals, operation_result_ty) = if lhs_ty.is_matrix() || rhs_ty.is_matrix() {
            // Use matrix operations for matrix compound assignments
            matrix::translate_matrix_binary(self, &binary_op, lhs_vals, &lhs_ty, rhs_vals, &rhs_ty, rhs_span.clone())?
        } else if lhs_ty.is_vector() || rhs_ty.is_vector() {
            // Use vector operations
            vector::translate_vector_binary(
                self,
                &binary_op,
                lhs_vals,
                &lhs_ty,
                rhs_vals,
                &rhs_ty,
                Some(rhs_span.clone()),
            )?
        } else {
            // Use scalar operations - need to determine base type for coercion
            let base_ty = if lhs_ty.is_numeric() && rhs_ty.is_numeric() {
                use crate::semantic::type_check::promote_numeric;
                promote_numeric(&lhs_ty, &rhs_ty)
            } else {
                lhs_ty.clone()
            };
            
            // Coerce operands to common type
            let lhs_val_coerced = coercion::coerce_to_type(self, lhs_vals[0], &lhs_ty, &base_ty)?;
            let rhs_val_coerced = coercion::coerce_to_type(self, rhs_vals[0], &rhs_ty, &base_ty)?;
            
            // Perform scalar operation
            let result_val = binary::translate_scalar_binary_op_internal(
                self,
                &binary_op,
                lhs_val_coerced,
                rhs_val_coerced,
                &base_ty,
                rhs_span.clone(),
            )?;
            
            // Result type is the same as the promoted type
            (vec![result_val], base_ty)
        };

        // Write result back to LHS
        write_lvalue(self, &lvalue, &operation_result_vals)?;

        // Return the result (same as simple assignment)
        let final_result = match &lvalue {
            crate::codegen::lvalue::LValue::Component { base_vars, base_ty, .. } => {
                // Component assignment returns the whole vector/matrix
                let mut result_vals = Vec::new();
                for &var in base_vars {
                    result_vals.push(self.builder.use_var(var));
                }
                (result_vals, base_ty.clone())
            }
            _ => {
                // Other assignments return the assigned values
                (operation_result_vals, operation_result_ty)
            }
        };

        Ok(final_result)
    }

    /// Coerce a value from one type to another (implements GLSL implicit conversions)
    pub fn coerce_to_type(
        &mut self,
        val: cranelift_codegen::ir::Value,
        from_ty: &GlslType,
        to_ty: &GlslType,
    ) -> Result<cranelift_codegen::ir::Value, GlslError> {
        coercion::coerce_to_type(self, val, from_ty, to_ty)
    }

    pub fn coerce_to_type_with_location(
        &mut self,
        val: cranelift_codegen::ir::Value,
        from_ty: &GlslType,
        to_ty: &GlslType,
        span: Option<glsl::syntax::SourceSpan>,
    ) -> Result<cranelift_codegen::ir::Value, GlslError> {
        coercion::coerce_to_type_with_location(self, val, from_ty, to_ty, span)
    }
}

