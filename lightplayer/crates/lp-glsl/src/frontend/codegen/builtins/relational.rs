//! Relational built-in functions for boolean vectors

use crate::error::GlslError;
use crate::frontend::codegen::context::CodegenContext;
use crate::semantic::types::Type;
use cranelift_codegen::ir::{condcodes::IntCC, types, InstBuilder, Value};

use alloc::vec::Vec;

impl<'a> CodegenContext<'a> {
    /// all(x) - returns true if all components are true
    pub fn builtin_all(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, _x_ty) = &args[0];

        let zero = self.builder.ins().iconst(types::I8, 0);
        let one = self.builder.ins().iconst(types::I8, 1);

        // AND reduction: all components must be non-zero
        let mut all_cmp: Option<Value> = None;
        for &val in x_vals {
            // Check if component is non-zero (returns I1)
            let is_nonzero = self.builder.ins().icmp(IntCC::NotEqual, val, zero);
            // AND with previous result (band works on I1)
            if let Some(prev) = all_cmp {
                all_cmp = Some(self.builder.ins().band(prev, is_nonzero));
            } else {
                all_cmp = Some(is_nonzero);
            }
        }

        let result_cmp = all_cmp.unwrap();
        let result = self.builder.ins().select(result_cmp, one, zero);

        Ok((vec![result], Type::Bool))
    }

    /// any(x) - returns true if any component is true
    pub fn builtin_any(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, _x_ty) = &args[0];

        let zero = self.builder.ins().iconst(types::I8, 0);
        let one = self.builder.ins().iconst(types::I8, 1);

        // OR reduction: any component must be non-zero
        let mut any_cmp: Option<Value> = None;
        for &val in x_vals {
            // Check if component is non-zero (returns I1)
            let is_nonzero = self.builder.ins().icmp(IntCC::NotEqual, val, zero);
            // OR with previous result (bor works on I1)
            if let Some(prev) = any_cmp {
                any_cmp = Some(self.builder.ins().bor(prev, is_nonzero));
            } else {
                any_cmp = Some(is_nonzero);
            }
        }

        let result_cmp = any_cmp.unwrap();
        let result = self.builder.ins().select(result_cmp, one, zero);

        Ok((vec![result], Type::Bool))
    }

    /// not(x) - component-wise logical NOT
    pub fn builtin_not(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let zero = self.builder.ins().iconst(types::I8, 0);
        let one = self.builder.ins().iconst(types::I8, 1);

        let mut result_vals = Vec::new();
        for &val in x_vals {
            // Logical NOT: if val == 0, return 1; else return 0
            let is_zero = self.builder.ins().icmp(IntCC::Equal, val, zero);
            let not_val = self.builder.ins().select(is_zero, one, zero);
            result_vals.push(not_val);
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// equal(x, y) - component-wise equality comparison
    pub fn builtin_equal(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        let base_ty = if x_ty.is_vector() {
            x_ty.vector_base_type().unwrap()
        } else {
            x_ty.clone()
        };

        let zero = self.builder.ins().iconst(types::I8, 0);
        let one = self.builder.ins().iconst(types::I8, 1);

        let mut result_vals = Vec::new();
        for i in 0..x_vals.len() {
            let cmp = if base_ty == Type::Bool || base_ty == Type::Int {
                self.builder.ins().icmp(IntCC::Equal, x_vals[i], y_vals[i])
            } else {
                self.builder.ins().fcmp(
                    cranelift_codegen::ir::condcodes::FloatCC::Equal,
                    x_vals[i],
                    y_vals[i],
                )
            };
            // Convert I1 to I8
            let result = self.builder.ins().select(cmp, one, zero);
            result_vals.push(result);
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// notEqual(x, y) - component-wise inequality comparison
    pub fn builtin_not_equal(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        let base_ty = if x_ty.is_vector() {
            x_ty.vector_base_type().unwrap()
        } else {
            x_ty.clone()
        };

        let zero = self.builder.ins().iconst(types::I8, 0);
        let one = self.builder.ins().iconst(types::I8, 1);

        let mut result_vals = Vec::new();
        for i in 0..x_vals.len() {
            let cmp = if base_ty == Type::Bool || base_ty == Type::Int {
                self.builder.ins().icmp(IntCC::NotEqual, x_vals[i], y_vals[i])
            } else {
                self.builder.ins().fcmp(
                    cranelift_codegen::ir::condcodes::FloatCC::NotEqual,
                    x_vals[i],
                    y_vals[i],
                )
            };
            // Convert I1 to I8
            let result = self.builder.ins().select(cmp, one, zero);
            result_vals.push(result);
        }

        Ok((result_vals, x_ty.clone()))
    }
}

