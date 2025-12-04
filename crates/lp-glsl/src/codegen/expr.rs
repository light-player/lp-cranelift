use cranelift_codegen::ir::{condcodes::IntCC, InstBuilder, Value};
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

impl<'a> CodegenContext<'a> {
    pub fn translate_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            // Literals
            Expr::IntConst(n) => {
                let val = self
                    .builder
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I32, *n as i64);
                Ok(val)
            }

            Expr::BoolConst(b) => {
                let val = self.builder.ins().iconst(
                    cranelift_codegen::ir::types::I8,
                    if *b { 1 } else { 0 },
                );
                Ok(val)
            }

            // Variable reference
            Expr::Variable(ident) => {
                let var = self
                    .lookup_variable(&ident.0)
                    .ok_or_else(|| format!("Variable '{}' not found", ident.0))?;
                let val = self.builder.use_var(var);
                Ok(val)
            }

            // Binary operators
            Expr::Binary(op, lhs, rhs) => {
                let lhs_val = self.translate_expr(lhs)?;
                let rhs_val = self.translate_expr(rhs)?;
                self.translate_binary_op(op, lhs_val, rhs_val)
            }

            // Unary operators
            Expr::Unary(op, expr) => {
                let val = self.translate_expr(expr)?;
                self.translate_unary_op(op, val)
            }

            // Assignment
            Expr::Assignment(lhs, op, rhs) => self.translate_assignment(lhs, op, rhs),

            _ => Err(format!("Expression not supported in Phase 1: {:?}", expr)),
        }
    }

    fn translate_binary_op(
        &mut self,
        op: &glsl::syntax::BinaryOp,
        lhs: Value,
        rhs: Value,
    ) -> Result<Value, String> {
        use glsl::syntax::BinaryOp::*;

        let val = match op {
            // Arithmetic
            Add => self.builder.ins().iadd(lhs, rhs),
            Sub => self.builder.ins().isub(lhs, rhs),
            Mult => self.builder.ins().imul(lhs, rhs),
            Div => self.builder.ins().sdiv(lhs, rhs),

            // Comparisons
            Equal => self.builder.ins().icmp(IntCC::Equal, lhs, rhs),
            NonEqual => self.builder.ins().icmp(IntCC::NotEqual, lhs, rhs),
            LT => self.builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs),
            GT => self.builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs),
            LTE => self
                .builder
                .ins()
                .icmp(IntCC::SignedLessThanOrEqual, lhs, rhs),
            GTE => self
                .builder
                .ins()
                .icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs),

            _ => return Err(format!("Binary operator not supported in Phase 1: {:?}", op)),
        };

        Ok(val)
    }

    fn translate_unary_op(
        &mut self,
        op: &glsl::syntax::UnaryOp,
        val: Value,
    ) -> Result<Value, String> {
        use glsl::syntax::UnaryOp::*;

        let result = match op {
            Minus => self.builder.ins().ineg(val),
            Not => {
                let zero = self
                    .builder
                    .ins()
                    .iconst(cranelift_codegen::ir::types::I8, 0);
                self.builder.ins().icmp(IntCC::Equal, val, zero)
            }
            _ => return Err(format!("Unary operator not supported in Phase 1: {:?}", op)),
        };

        Ok(result)
    }

    fn translate_assignment(
        &mut self,
        lhs: &Expr,
        op: &glsl::syntax::AssignmentOp,
        rhs: &Expr,
    ) -> Result<Value, String> {
        // Phase 1: Only simple assignment (=)
        if !matches!(op, glsl::syntax::AssignmentOp::Equal) {
            return Err("Only simple assignment (=) supported in Phase 1".to_string());
        }

        // Get variable name from lhs
        let var_name = match lhs {
            Expr::Variable(ident) => &ident.0,
            _ => return Err("Assignment lhs must be variable in Phase 1".to_string()),
        };

        let var = self
            .lookup_variable(var_name)
            .ok_or_else(|| format!("Variable '{}' not found", var_name))?;

        let rhs_val = self.translate_expr(rhs)?;
        self.builder.def_var(var, rhs_val);

        Ok(rhs_val)
    }
}

