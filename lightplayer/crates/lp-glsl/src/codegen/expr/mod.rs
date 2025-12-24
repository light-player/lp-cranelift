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
use crate::semantic::types::Type as GlslType;
use crate::error::{ErrorCode, GlslError, source_span_to_location};
use glsl::syntax::Expr;
use cranelift_codegen::ir::Value;

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

impl<'a> CodegenContext<'a> {
    /// Main entry point for expression translation
    pub fn translate_expr_typed(&mut self, expr: &Expr) -> Result<(Vec<Value>, GlslType), GlslError> {
        match expr {
            Expr::IntConst(..) | Expr::FloatConst(..) | Expr::BoolConst(..) => {
                literal::translate_literal(self, expr)
            }
            Expr::Variable(..) => {
                variable::translate_variable(self, expr)
            }
            Expr::Binary(..) => {
                binary::translate_binary(self, expr)
            }
            Expr::Unary(op, operand, span) => {
                // Handle pre-increment/decrement specially
                use glsl::syntax::UnaryOp::*;
                match op {
                    Inc => {
                        incdec::translate_preinc(self, operand, span.clone())
                    }
                    Dec => {
                        incdec::translate_predec(self, operand, span.clone())
                    }
                    _ => unary::translate_unary(self, expr),
                }
            }
            Expr::FunCall(..) => {
                function::translate_function_call(self, expr)
            }
            Expr::Dot(..) => {
                component::translate_component_access(self, expr)
            }
            Expr::Bracket(..) => {
                // Matrix indexing - handled in component module
                component::translate_matrix_indexing(self, expr)
            }
            Expr::Assignment(lhs, op, rhs, _span) => {
                // Assignment is handled in stmt.rs, but expression result
                // needs to be computed here
                self.translate_assignment_typed(lhs, op, rhs)
            }
            Expr::PostInc(operand, span) => {
                incdec::translate_postinc(self, operand, span.clone())
            }
            Expr::PostDec(operand, span) => {
                incdec::translate_postdec(self, operand, span.clone())
            }
            _ => Err(GlslError::new(ErrorCode::E0400, format!("expression not supported yet: {:?}", expr))),
        }
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
        use crate::codegen::lvalue::{resolve_lvalue, write_lvalue};
        use crate::error::extract_span_from_expr;
        use crate::error::extract_span_from_identifier;
        use crate::semantic::type_check::check_assignment;
        use component;
        
        // Only simple assignment (=) for now
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return Err(GlslError::new(ErrorCode::E0400, "only simple assignment (=) supported"));
        }

        // Resolve LHS to an LValue
        let lvalue = resolve_lvalue(self, lhs)?;
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
        let expected_count = match &lvalue {
            crate::codegen::lvalue::LValue::Variable { vars, .. } => vars.len(),
            crate::codegen::lvalue::LValue::Component { indices, .. } => indices.len(),
            crate::codegen::lvalue::LValue::MatrixElement { .. } => 1,
            crate::codegen::lvalue::LValue::MatrixColumn { result_ty, .. } => {
                result_ty.component_count().unwrap()
            }
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

