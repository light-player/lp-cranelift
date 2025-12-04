use cranelift_codegen::ir::{condcodes::{IntCC, FloatCC}, types, InstBuilder, Value};
use glsl::syntax::Expr;

#[cfg(feature = "std")]
use std::string::{String, ToString};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};

#[cfg(feature = "std")]
use std::format;
#[cfg(not(feature = "std"))]
use alloc::format;

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type as GlslType;
use crate::semantic::type_check::{infer_binary_result_type, infer_unary_result_type, promote_numeric, check_assignment, check_vector_constructor, is_vector_type_name};

#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

impl<'a> CodegenContext<'a> {
    /// Translate expression and return values (vec for vectors, single element for scalars) and type
    pub fn translate_expr_typed(&mut self, expr: &Expr) -> Result<(Vec<Value>, GlslType), String> {
        match expr {
            // Literals - scalars return single-element vec
            Expr::IntConst(n) => {
                let val = self.builder.ins().iconst(types::I32, *n as i64);
                Ok((vec![val], GlslType::Int))
            }

            Expr::FloatConst(f) => {
                let val = self.builder.ins().f32const(*f);
                Ok((vec![val], GlslType::Float))
            }

            Expr::BoolConst(b) => {
                let val = self.builder.ins().iconst(types::I8, if *b { 1 } else { 0 });
                Ok((vec![val], GlslType::Bool))
            }

            // Variable reference - returns all components
            Expr::Variable(ident) => {
                let vars = self
                    .lookup_variables(&ident.0)
                    .ok_or_else(|| format!("Variable '{}' not found", ident.0))?;
                let ty = self
                    .lookup_variable_type(&ident.0)
                    .ok_or_else(|| format!("Variable type not found for '{}'", ident.0))?
                    .clone();
                
                let vals: Vec<Value> = vars.iter()
                    .map(|&v| self.builder.use_var(v))
                    .collect();
                
                Ok((vals, ty))
            }

            // Function calls - check if it's a type constructor
            Expr::FunCall(func_ident, args) => {
                if is_vector_type_name(&func_ident.ident.0) {
                    self.translate_vector_constructor(&func_ident.ident.0, args)
                } else {
                    // Regular function call (not implemented yet)
                    Err(format!("Function calls not yet supported: {}", func_ident.ident.0))
                }
            }

            // Binary operators - scalars only for now
            Expr::Binary(op, lhs, rhs) => {
                let (lhs_vals, lhs_ty) = self.translate_expr_typed(lhs)?;
                let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;

                // Only scalars supported in binary ops for now
                if lhs_vals.len() != 1 || rhs_vals.len() != 1 {
                    return Err("Vector arithmetic not yet implemented".to_string());
                }

                let lhs_val = lhs_vals[0];
                let rhs_val = rhs_vals[0];

                // Infer result type and validate
                let result_ty = infer_binary_result_type(op, &lhs_ty, &rhs_ty)?;

                // Promote operands to common type (for arithmetic ops)
                let common_ty = promote_numeric(&lhs_ty, &rhs_ty);
                let lhs_val = self.coerce_to_type(lhs_val, &lhs_ty, &common_ty)?;
                let rhs_val = self.coerce_to_type(rhs_val, &rhs_ty, &common_ty)?;

                // Generate operation
                let result_val = self.translate_binary_op(op, lhs_val, rhs_val, &common_ty)?;
                Ok((vec![result_val], result_ty))
            }

            // Unary operators - scalars only for now
            Expr::Unary(op, expr) => {
                let (vals, ty) = self.translate_expr_typed(expr)?;
                
                if vals.len() != 1 {
                    return Err("Vector unary ops not yet implemented".to_string());
                }

                let val = vals[0];
                let result_ty = infer_unary_result_type(op, &ty)?;
                let result_val = self.translate_unary_op(op, val, &ty)?;
                Ok((vec![result_val], result_ty))
            }

            // Assignment
            Expr::Assignment(lhs, op, rhs) => self.translate_assignment_typed(lhs, op, rhs),

            _ => Err(format!("Expression not supported yet: {:?}", expr)),
        }
    }

    /// Legacy wrapper for compatibility - returns just the first value (for scalars)
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        let (vals, _ty) = self.translate_expr_typed(expr)?;
        vals.into_iter().next().ok_or_else(|| "Expression produced no values".to_string())
    }

    fn translate_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs: Value,
        rhs: Value,
        operand_ty: &GlslType,
    ) -> Result<Value, String> {
        use glsl::syntax::BinaryOp::*;

        let val = match op {
            // Arithmetic operators - dispatch based on type
            Add => match operand_ty {
                GlslType::Int => self.builder.ins().iadd(lhs, rhs),
                GlslType::Float => self.builder.ins().fadd(lhs, rhs),
                _ => return Err(format!("Add not supported for {:?}", operand_ty)),
            },
            Sub => match operand_ty {
                GlslType::Int => self.builder.ins().isub(lhs, rhs),
                GlslType::Float => self.builder.ins().fsub(lhs, rhs),
                _ => return Err(format!("Sub not supported for {:?}", operand_ty)),
            },
            Mult => match operand_ty {
                GlslType::Int => self.builder.ins().imul(lhs, rhs),
                GlslType::Float => self.builder.ins().fmul(lhs, rhs),
                _ => return Err(format!("Mult not supported for {:?}", operand_ty)),
            },
            Div => match operand_ty {
                GlslType::Int => self.builder.ins().sdiv(lhs, rhs),
                GlslType::Float => self.builder.ins().fdiv(lhs, rhs),
                _ => return Err(format!("Div not supported for {:?}", operand_ty)),
            },

            // Comparison operators - dispatch based on type
            Equal => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::Equal, lhs, rhs),
                _ => return Err(format!("Equal not supported for {:?}", operand_ty)),
            },
            NonEqual => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::NotEqual, lhs, rhs),
                _ => return Err(format!("NonEqual not supported for {:?}", operand_ty)),
            },
            LT => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::LessThan, lhs, rhs),
                _ => return Err(format!("LT not supported for {:?}", operand_ty)),
            },
            GT => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::GreaterThan, lhs, rhs),
                _ => return Err(format!("GT not supported for {:?}", operand_ty)),
            },
            LTE => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::LessThanOrEqual, lhs, rhs),
                _ => return Err(format!("LTE not supported for {:?}", operand_ty)),
            },
            GTE => match operand_ty {
                GlslType::Int => self.builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),
                GlslType::Float => self.builder.ins().fcmp(FloatCC::GreaterThanOrEqual, lhs, rhs),
                _ => return Err(format!("GTE not supported for {:?}", operand_ty)),
            },

            // Logical operators (bool only, already validated by type_check)
            And | Or | Xor => {
                return Err(format!("Logical operator {:?} not yet implemented", op));
            }

            _ => return Err(format!("Binary operator not supported yet: {:?}", op)),
        };

        Ok(val)
    }

    fn translate_unary_op(
        &mut self,
        op: &glsl::syntax::UnaryOp,
        val: Value,
        operand_ty: &GlslType,
    ) -> Result<Value, String> {
        use glsl::syntax::UnaryOp::*;

        let result = match op {
            Minus => match operand_ty {
                GlslType::Int => self.builder.ins().ineg(val),
                GlslType::Float => self.builder.ins().fneg(val),
                _ => return Err(format!("Unary minus not supported for {:?}", operand_ty)),
            },
            Not => {
                if operand_ty != &GlslType::Bool {
                    return Err(format!("Logical NOT requires bool, got {:?}", operand_ty));
                }
                let zero = self.builder.ins().iconst(types::I8, 0);
                self.builder.ins().icmp(IntCC::Equal, val, zero)
            }
            _ => return Err(format!("Unary operator not supported yet: {:?}", op)),
        };

        Ok(result)
    }

    /// Coerce a value from one type to another (implements GLSL implicit conversions)
    fn coerce_to_type(
        &mut self,
        val: Value,
        from_ty: &GlslType,
        to_ty: &GlslType,
    ) -> Result<Value, String> {
        if from_ty == to_ty {
            return Ok(val);
        }

        match (from_ty, to_ty) {
            (GlslType::Int, GlslType::Float) => {
                // int → float: fcvt_from_sint
                Ok(self.builder.ins().fcvt_from_sint(types::F32, val))
            }
            _ => Err(format!("Cannot implicitly convert {:?} to {:?}", from_ty, to_ty)),
        }
    }

    fn translate_assignment_typed(
        &mut self,
        lhs: &Expr,
        op: &glsl::syntax::AssignmentOp,
        rhs: &Expr,
    ) -> Result<(Vec<Value>, GlslType), String> {
        // Only simple assignment (=) for now
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return Err("Only simple assignment (=) supported".to_string());
        }

        // Get variable name from lhs
        let var_name = match lhs {
            Expr::Variable(ident) => &ident.0,
            _ => return Err("Assignment lhs must be variable".to_string()),
        };

        let vars = self
            .lookup_variables(var_name)
            .ok_or_else(|| format!("Variable '{}' not found", var_name))?;

        let lhs_ty = self
            .lookup_variable_type(var_name)
            .ok_or_else(|| format!("Variable type not found for '{}'", var_name))?
            .clone();

        // Translate RHS
        let (rhs_vals, rhs_ty) = self.translate_expr_typed(rhs)?;

        // Validate assignment (check implicit conversion is allowed)
        check_assignment(&lhs_ty, &rhs_ty)?;

        // Check component counts match
        if vars.len() != rhs_vals.len() {
            return Err(format!(
                "Component count mismatch in assignment: {} vs {}",
                vars.len(), rhs_vals.len()
            ));
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

    fn translate_vector_constructor(
        &mut self,
        type_name: &str,
        args: &[Expr],
    ) -> Result<(Vec<Value>, GlslType), String> {
        // Translate all arguments
        let mut arg_vals: Vec<Vec<Value>> = Vec::new();
        let mut arg_types: Vec<GlslType> = Vec::new();
        
        for arg in args {
            let (vals, ty) = self.translate_expr_typed(arg)?;
            arg_vals.push(vals);
            arg_types.push(ty);
        }

        // Type check constructor
        let result_type = check_vector_constructor(type_name, &arg_types)?;
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
}

