//! Code generation for GLSL built-in functions

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type;
use cranelift_codegen::ir::{InstBuilder, Value, condcodes::IntCC, types};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use std::string::{String, ToString};

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

impl<'a> CodegenContext<'a> {
    pub fn translate_builtin_call(
        &mut self,
        name: &str,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        match name {
            "dot" => self.builtin_dot(args),
            "cross" => self.builtin_cross(args),
            "length" => self.builtin_length(args),
            "normalize" => self.builtin_normalize(args),
            "distance" => self.builtin_distance(args),
            "min" => self.builtin_min(args),
            "max" => self.builtin_max(args),
            "clamp" => self.builtin_clamp(args),
            "abs" => self.builtin_abs(args),
            "sqrt" => self.builtin_sqrt(args),
            "floor" => self.builtin_floor(args),
            "ceil" => self.builtin_ceil(args),
            "pow" => self.builtin_pow(args),
            _ => Err(format!("Built-in function not implemented: {}", name)),
        }
    }

    // Geometric Functions

    /// Dot product: x·y = x₀y₀ + x₁y₁ + x₂y₂ + ...
    fn builtin_dot(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, _) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != y_vals.len() {
            return Err("dot() requires matching vector sizes".to_string());
        }

        let mut sum = self.builder.ins().fmul(x_vals[0], y_vals[0]);
        for i in 1..x_vals.len() {
            let product = self.builder.ins().fmul(x_vals[i], y_vals[i]);
            sum = self.builder.ins().fadd(sum, product);
        }

        Ok((vec![sum], Type::Float))
    }

    /// Cross product: x × y (vec3 only)
    fn builtin_cross(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, _) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != 3 || y_vals.len() != 3 {
            return Err("cross() requires vec3 arguments".to_string());
        }

        // cross(x, y) = (x.y*y.z - x.z*y.y, x.z*y.x - x.x*y.z, x.x*y.y - x.y*y.x)
        let x0 = x_vals[0];
        let x1 = x_vals[1];
        let x2 = x_vals[2];
        let y0 = y_vals[0];
        let y1 = y_vals[1];
        let y2 = y_vals[2];

        let r0 = {
            let a = self.builder.ins().fmul(x1, y2);
            let b = self.builder.ins().fmul(x2, y1);
            self.builder.ins().fsub(a, b)
        };
        let r1 = {
            let a = self.builder.ins().fmul(x2, y0);
            let b = self.builder.ins().fmul(x0, y2);
            self.builder.ins().fsub(a, b)
        };
        let r2 = {
            let a = self.builder.ins().fmul(x0, y1);
            let b = self.builder.ins().fmul(x1, y0);
            self.builder.ins().fsub(a, b)
        };

        Ok((vec![r0, r1, r2], Type::Vec3))
    }

    /// length(x) = sqrt(dot(x, x))
    fn builtin_length(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, _) = &args[0];

        // Compute dot product with self
        let mut sum_sq = self.builder.ins().fmul(x_vals[0], x_vals[0]);
        for i in 1..x_vals.len() {
            let sq = self.builder.ins().fmul(x_vals[i], x_vals[i]);
            sum_sq = self.builder.ins().fadd(sum_sq, sq);
        }

        // Square root
        let result = self.builder.ins().sqrt(sum_sq);

        Ok((vec![result], Type::Float))
    }

    /// normalize(x) = x / length(x)
    fn builtin_normalize(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, x_ty) = &args[0];

        // Compute length
        let (len_val, _) = self.builtin_length(vec![(x_vals.clone(), x_ty.clone())])?;
        let len = len_val[0];

        // Divide each component by length
        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().fdiv(val, len));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// distance(p0, p1) = length(p0 - p1)
    fn builtin_distance(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (p0_vals, p0_ty) = &args[0];
        let (p1_vals, _) = &args[1];

        if p0_vals.len() != p1_vals.len() {
            return Err("distance() requires matching vector sizes".to_string());
        }

        // Compute p0 - p1
        let mut diff_vals = Vec::new();
        for i in 0..p0_vals.len() {
            diff_vals.push(self.builder.ins().fsub(p0_vals[i], p1_vals[i]));
        }

        // Compute length of difference
        self.builtin_length(vec![(diff_vals, p0_ty.clone())])
    }

    // Common Functions

    /// min(x, y) - component-wise for vectors
    fn builtin_min(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), String> {
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
                    _ => return Err("min() not supported for this type".to_string()),
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
                    _ => return Err("min() not supported for this type".to_string()),
                };
                result_vals.push(min_val);
            }
        }

        Ok((result_vals, result_ty))
    }

    /// max(x, y) - component-wise for vectors
    fn builtin_max(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), String> {
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
                    _ => return Err("max() not supported for this type".to_string()),
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
                    _ => return Err("max() not supported for this type".to_string()),
                };
                result_vals.push(max_val);
            }
        }

        Ok((result_vals, result_ty))
    }

    /// clamp(x, minVal, maxVal) = min(max(x, minVal), maxVal)
    fn builtin_clamp(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let x_arg = args[0].clone();
        let min_arg = args[1].clone();
        let max_arg = args[2].clone();

        // First: max(x, minVal)
        let (temp_vals, temp_ty) = self.builtin_max(vec![x_arg, min_arg])?;

        // Then: min(temp, maxVal)
        self.builtin_min(vec![(temp_vals, temp_ty), max_arg])
    }

    /// abs(x) - absolute value
    fn builtin_abs(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), String> {
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
                _ => return Err("abs() not supported for this type".to_string()),
            };
            result_vals.push(abs_val);
        }

        Ok((result_vals, x_ty.clone()))
    }

    // Math Functions

    /// sqrt(x) - square root (component-wise)
    fn builtin_sqrt(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().sqrt(val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// floor(x) - round down to nearest integer (component-wise)
    fn builtin_floor(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().floor(val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// ceil(x) - round up to nearest integer (component-wise)
    fn builtin_ceil(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, x_ty) = &args[0];

        let mut result_vals = Vec::new();
        for &val in x_vals {
            result_vals.push(self.builder.ins().ceil(val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// pow(x, y) = x^y (component-wise)
    fn builtin_pow(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), String> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != y_vals.len() {
            return Err("pow() requires matching sizes".to_string());
        }

        // TODO: Cranelift doesn't have fpow instruction - need to implement via exp/log
        // For now, return error
        Err("pow() builtin not yet implemented (needs exp/log)".to_string())
    }
}
