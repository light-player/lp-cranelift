//! Common built-in functions

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{InstBuilder, Value, condcodes::IntCC, types};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

impl<'a> CodegenContext<'a> {
    /// min(x, y) - component-wise for vectors
    pub fn builtin_min(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        let result_ty = x_ty.clone();
        let base_ty = if x_ty.is_vector() {
            x_ty.vector_base_type().unwrap()
        } else {
            x_ty.clone()
        };

        let mut result_vals = Vec::new();

        // Handle scalar broadcast (min(vec3, float))
        if x_vals.len() > 1 && y_vals.len() == 1 {
            let y_scalar = y_vals[0];
            for &x in x_vals {
                let min_val = match base_ty {
                    Type::Float => self.builder.ins().fmin(x, y_scalar),
                    Type::Int => {
                        let cmp = self.builder.ins().icmp(IntCC::SignedLessThan, x, y_scalar);
                        self.builder.ins().select(cmp, x, y_scalar)
                    }
                    _ => return Err(GlslError::new(ErrorCode::E0105, "min() not supported for this type")),
                };
                result_vals.push(min_val);
            }
        } else {
            // Component-wise min
            for i in 0..x_vals.len() {
                let min_val = match base_ty {
                    Type::Float => self.builder.ins().fmin(x_vals[i], y_vals[i]),
                    Type::Int => {
                        let cmp =
                            self.builder
                                .ins()
                                .icmp(IntCC::SignedLessThan, x_vals[i], y_vals[i]);
                        self.builder.ins().select(cmp, x_vals[i], y_vals[i])
                    }
                    _ => return Err(GlslError::new(ErrorCode::E0105, "min() not supported for this type")),
                };
                result_vals.push(min_val);
            }
        }

        Ok((result_vals, result_ty))
    }

    /// max(x, y) - component-wise for vectors
    pub fn builtin_max(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        let result_ty = x_ty.clone();
        let base_ty = if x_ty.is_vector() {
            x_ty.vector_base_type().unwrap()
        } else {
            x_ty.clone()
        };

        let mut result_vals = Vec::new();

        // Handle scalar broadcast (max(vec3, float))
        if x_vals.len() > 1 && y_vals.len() == 1 {
            let y_scalar = y_vals[0];
            for &x in x_vals {
                let max_val = match base_ty {
                    Type::Float => self.builder.ins().fmax(x, y_scalar),
                    Type::Int => {
                        let cmp = self
                            .builder
                            .ins()
                            .icmp(IntCC::SignedGreaterThan, x, y_scalar);
                        self.builder.ins().select(cmp, x, y_scalar)
                    }
                    _ => return Err(GlslError::new(ErrorCode::E0105, "max() not supported for this type")),
                };
                result_vals.push(max_val);
            }
        } else {
            // Component-wise max
            for i in 0..x_vals.len() {
                let max_val = match base_ty {
                    Type::Float => self.builder.ins().fmax(x_vals[i], y_vals[i]),
                    Type::Int => {
                        let cmp =
                            self.builder
                                .ins()
                                .icmp(IntCC::SignedGreaterThan, x_vals[i], y_vals[i]);
                        self.builder.ins().select(cmp, x_vals[i], y_vals[i])
                    }
                    _ => return Err(GlslError::new(ErrorCode::E0105, "max() not supported for this type")),
                };
                result_vals.push(max_val);
            }
        }

        Ok((result_vals, result_ty))
    }

    /// clamp(x, minVal, maxVal) = min(max(x, minVal), maxVal)
    pub fn builtin_clamp(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let x_arg = args[0].clone();
        let min_arg = args[1].clone();
        let max_arg = args[2].clone();

        // First: max(x, minVal)
        let (temp_vals, temp_ty) = self.builtin_max(vec![x_arg, min_arg])?;

        // Then: min(temp, maxVal)
        self.builtin_min(vec![(temp_vals, temp_ty), max_arg])
    }

    /// abs(x) - absolute value
    pub fn builtin_abs(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let base_ty = if x_ty.is_vector() {
            x_ty.vector_base_type().unwrap()
        } else {
            x_ty.clone()
        };

        let mut result_vals = Vec::new();
        for &val in x_vals {
            let abs_val = match base_ty {
                Type::Float => self.builder.ins().fabs(val),
                Type::Int => {
                    // abs for int: (x < 0) ? -x : x
                    let zero = self.builder.ins().iconst(types::I32, 0);
                    let is_neg = self.builder.ins().icmp(IntCC::SignedLessThan, val, zero);
                    let neg_val = self.builder.ins().ineg(val);
                    self.builder.ins().select(is_neg, neg_val, val)
                }
                _ => return Err(GlslError::new(ErrorCode::E0105, "abs() not supported for this type")),
            };
            result_vals.push(abs_val);
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// sqrt(x) - square root (component-wise)
    pub fn builtin_sqrt(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().sqrt(val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// floor(x) - round down to nearest integer (component-wise)
    pub fn builtin_floor(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().floor(val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// ceil(x) - round up to nearest integer (component-wise)
    pub fn builtin_ceil(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().ceil(val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// pow(x, y) = x^y (component-wise)
    pub fn builtin_pow(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, _x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != y_vals.len() {
            return Err(GlslError::new(ErrorCode::E0104, "pow() requires matching sizes"));
        }

        // TODO: Cranelift doesn't have fpow instruction - need to implement via exp/log
        // For now, return error
        Err(GlslError::new(ErrorCode::E0400, "pow() builtin not yet implemented (needs exp/log)"))
    }

    /// fract(x) = x - floor(x) (fractional part)
    pub fn builtin_fract(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            let floored = self.builder.ins().floor(val);
            result_vals.push(self.builder.ins().fsub(val, floored));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// mod(x, y) = x - y * floor(x/y)
    pub fn builtin_mod(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        let mut result_vals = Vec::new();

        // Handle scalar broadcast (mod(vec3, float))
        if x_vals.len() > 1 && y_vals.len() == 1 {
            let y_scalar = y_vals[0];
            for &x in x_vals {
                // floor(x / y)
                let div = self.builder.ins().fdiv(x, y_scalar);
                let floored = self.builder.ins().floor(div);
                // y * floor(x / y)
                let y_times_floor = self.builder.ins().fmul(y_scalar, floored);
                // x - y * floor(x / y)
                result_vals.push(self.builder.ins().fsub(x, y_times_floor));
            }
        } else {
            // Component-wise mod
            for i in 0..x_vals.len() {
                // floor(x / y)
                let div = self.builder.ins().fdiv(x_vals[i], y_vals[i]);
                let floored = self.builder.ins().floor(div);
                // y * floor(x / y)
                let y_times_floor = self.builder.ins().fmul(y_vals[i], floored);
                // x - y * floor(x / y)
                result_vals.push(self.builder.ins().fsub(x_vals[i], y_times_floor));
            }
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// sign(x) - returns -1.0, 0.0, or 1.0 based on sign of x
    pub fn builtin_sign(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];

        let base_ty = if x_ty.is_vector() {
            x_ty.vector_base_type().unwrap()
        } else {
            x_ty.clone()
        };

        let mut result_vals = Vec::new();

        match base_ty {
            Type::Float => {
                let zero = self.builder.ins().f32const(0.0);
                let one = self.builder.ins().f32const(1.0);
                let minus_one = self.builder.ins().f32const(-1.0);

                for &val in x_vals {
                    // Check if x > 0
                    let gt_zero = self.builder.ins().fcmp(
                        cranelift_codegen::ir::condcodes::FloatCC::GreaterThan,
                        val,
                        zero,
                    );
                    // Check if x < 0
                    let lt_zero = self.builder.ins().fcmp(
                        cranelift_codegen::ir::condcodes::FloatCC::LessThan,
                        val,
                        zero,
                    );

                    // If x > 0, return 1.0, else continue
                    let temp = self.builder.ins().select(gt_zero, one, zero);
                    // If x < 0, return -1.0, else use previous result
                    let result = self.builder.ins().select(lt_zero, minus_one, temp);

                    result_vals.push(result);
                }
            }
            Type::Int => {
                let zero = self.builder.ins().iconst(types::I32, 0);
                let one = self.builder.ins().iconst(types::I32, 1);
                let minus_one = self.builder.ins().iconst(types::I32, -1);

                for &val in x_vals {
                    // Check if x > 0
                    let gt_zero = self.builder.ins().icmp(IntCC::SignedGreaterThan, val, zero);
                    // Check if x < 0
                    let lt_zero = self.builder.ins().icmp(IntCC::SignedLessThan, val, zero);

                    // If x > 0, return 1, else continue
                    let temp = self.builder.ins().select(gt_zero, one, zero);
                    // If x < 0, return -1, else use previous result
                    let result = self.builder.ins().select(lt_zero, minus_one, temp);

                    result_vals.push(result);
                }
            }
            _ => return Err(GlslError::new(ErrorCode::E0105, "sign() not supported for this type")),
        }

        Ok((result_vals, x_ty.clone()))
    }
}





