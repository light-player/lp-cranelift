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
            Expr::Unary(..) => {
                unary::translate_unary(self, expr)
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
        use crate::error::extract_span_from_expr;
        use crate::semantic::type_check::check_assignment;
        
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
            let coerced = coercion::coerce_to_type(self, *val, &rhs_base, &lhs_base)?;
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
        use crate::error::extract_span_from_expr;
        use crate::error::extract_span_from_identifier;
        use crate::semantic::type_check::check_assignment;
        
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
        let indices = component::parse_vector_swizzle(&field.name, &vec_ty, Some(field_span))?;
        let base_ty = vec_ty.vector_base_type().unwrap();

        // Check for duplicates (illegal in assignment LHS)
        if component::has_duplicates(&indices) {
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
            let rhs_val = coercion::coerce_to_type(self, rhs_vals[i], &rhs_base, &base_ty)?;
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

