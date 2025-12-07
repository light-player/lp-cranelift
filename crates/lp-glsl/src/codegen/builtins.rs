//! Code generation for GLSL built-in functions

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{InstBuilder, Value, condcodes::IntCC, types};

#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(feature = "std")]
use std::vec::Vec;

#[cfg(not(feature = "std"))]
use alloc::format;
#[cfg(feature = "std")]
use std::format;

impl<'a> CodegenContext<'a> {
    pub fn translate_builtin_call(
        &mut self,
        name: &str,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
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
            "mix" => self.builtin_mix(args),
            "step" => self.builtin_step(args),
            "smoothstep" => self.builtin_smoothstep(args),
            "fract" => self.builtin_fract(args),
            "mod" => self.builtin_mod(args),
            "sign" => self.builtin_sign(args),
            // Matrix Functions (builtinfunctions.adoc:1538-1687)
            "matrixCompMult" => self.builtin_matrixCompMult(args),
            "outerProduct" => self.builtin_outerProduct(args),
            "transpose" => self.builtin_transpose(args),
            "determinant" => self.builtin_determinant(args),
            "inverse" => self.builtin_inverse(args),
            _ => Err(GlslError::new(ErrorCode::E0400, format!("built-in function not implemented: {}", name))),
        }
    }

    // Geometric Functions

    /// Dot product: x·y = x₀y₀ + x₁y₁ + x₂y₂ + ...
    fn builtin_dot(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, _) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != y_vals.len() {
            return Err(GlslError::new(ErrorCode::E0104, "dot() requires matching vector sizes"));
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
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, _) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != 3 || y_vals.len() != 3 {
            return Err(GlslError::new(ErrorCode::E0104, "cross() requires vec3 arguments"));
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
    ) -> Result<(Vec<Value>, Type), GlslError> {
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
    ) -> Result<(Vec<Value>, Type), GlslError> {
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
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (p0_vals, p0_ty) = &args[0];
        let (p1_vals, _) = &args[1];

        if p0_vals.len() != p1_vals.len() {
            return Err(GlslError::new(ErrorCode::E0104, "distance() requires matching vector sizes"));
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
    fn builtin_min(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
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
    fn builtin_max(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
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
    fn builtin_clamp(
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
    fn builtin_abs(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
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

    // Math Functions

    /// sqrt(x) - square root (component-wise)
    fn builtin_sqrt(
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
    fn builtin_floor(
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
    fn builtin_ceil(
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
    fn builtin_pow(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, _x_ty) = &args[0];
        let (y_vals, _) = &args[1];

        if x_vals.len() != y_vals.len() {
            return Err(GlslError::new(ErrorCode::E0104, "pow() requires matching sizes"));
        }

        // TODO: Cranelift doesn't have fpow instruction - need to implement via exp/log
        // For now, return error
        Err(GlslError::new(ErrorCode::E0400, "pow() builtin not yet implemented (needs exp/log)"))
    }

    /// mix(x, y, a) = x * (1-a) + y * a (linear interpolation)
    fn builtin_mix(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, _) = &args[1];
        let (a_vals, _) = &args[2];

        let mut result_vals = Vec::new();

        // Handle scalar broadcast (mix(vec3, vec3, float))
        if x_vals.len() > 1 && a_vals.len() == 1 {
            let a_scalar = a_vals[0];
            // Compute (1 - a)
            let one = self.builder.ins().f32const(1.0);
            let one_minus_a = self.builder.ins().fsub(one, a_scalar);

            for i in 0..x_vals.len() {
                // x * (1-a)
                let x_part = self.builder.ins().fmul(x_vals[i], one_minus_a);
                // y * a
                let y_part = self.builder.ins().fmul(y_vals[i], a_scalar);
                // x * (1-a) + y * a
                result_vals.push(self.builder.ins().fadd(x_part, y_part));
            }
        } else {
            // Component-wise mix
            for i in 0..x_vals.len() {
                // (1 - a)
                let one = self.builder.ins().f32const(1.0);
                let one_minus_a = self.builder.ins().fsub(one, a_vals[i]);
                // x * (1-a)
                let x_part = self.builder.ins().fmul(x_vals[i], one_minus_a);
                // y * a
                let y_part = self.builder.ins().fmul(y_vals[i], a_vals[i]);
                // x * (1-a) + y * a
                result_vals.push(self.builder.ins().fadd(x_part, y_part));
            }
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// step(edge, x) = x < edge ? 0.0 : 1.0
    fn builtin_step(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (edge_vals, _) = &args[0];
        let (x_vals, x_ty) = &args[1];

        let mut result_vals = Vec::new();
        let zero = self.builder.ins().f32const(0.0);
        let one = self.builder.ins().f32const(1.0);

        // Handle scalar broadcast (step(float, vec3))
        if edge_vals.len() == 1 && x_vals.len() > 1 {
            let edge_scalar = edge_vals[0];
            for &x in x_vals {
                // x < edge ? 0.0 : 1.0
                let cmp = self.builder.ins().fcmp(
                    cranelift_codegen::ir::condcodes::FloatCC::LessThan,
                    x,
                    edge_scalar,
                );
                result_vals.push(self.builder.ins().select(cmp, zero, one));
            }
        } else {
            // Component-wise step
            for i in 0..x_vals.len() {
                let cmp = self.builder.ins().fcmp(
                    cranelift_codegen::ir::condcodes::FloatCC::LessThan,
                    x_vals[i],
                    edge_vals[i],
                );
                result_vals.push(self.builder.ins().select(cmp, zero, one));
            }
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// smoothstep(edge0, edge1, x) - Smooth Hermite interpolation
    /// Formula: t = clamp((x - edge0) / (edge1 - edge0), 0, 1); return t * t * (3 - 2 * t);
    fn builtin_smoothstep(
        &mut self,
        args: Vec<(Vec<Value>, Type)>,
    ) -> Result<(Vec<Value>, Type), GlslError> {
        let (edge0_vals, _) = &args[0];
        let (edge1_vals, _) = &args[1];
        let (x_vals, x_ty) = &args[2];

        let mut result_vals = Vec::new();
        let zero = self.builder.ins().f32const(0.0);
        let one = self.builder.ins().f32const(1.0);
        let two = self.builder.ins().f32const(2.0);
        let three = self.builder.ins().f32const(3.0);

        // Handle scalar broadcast (smoothstep(float, float, vec3))
        if edge0_vals.len() == 1 && x_vals.len() > 1 {
            let edge0_scalar = edge0_vals[0];
            let edge1_scalar = edge1_vals[0];

            for &x in x_vals {
                // t = (x - edge0) / (edge1 - edge0)
                let numerator = self.builder.ins().fsub(x, edge0_scalar);
                let denominator = self.builder.ins().fsub(edge1_scalar, edge0_scalar);
                let t_raw = self.builder.ins().fdiv(numerator, denominator);

                // t = clamp(t, 0, 1)
                let t_max = self.builder.ins().fmax(t_raw, zero);
                let t_clamped = self.builder.ins().fmin(t_max, one);

                // result = t * t * (3 - 2 * t)
                let t_squared = self.builder.ins().fmul(t_clamped, t_clamped);
                let two_t = self.builder.ins().fmul(two, t_clamped);
                let three_minus_two_t = self.builder.ins().fsub(three, two_t);
                let result = self.builder.ins().fmul(t_squared, three_minus_two_t);

                result_vals.push(result);
            }
        } else {
            // Component-wise smoothstep
            for i in 0..x_vals.len() {
                // t = (x - edge0) / (edge1 - edge0)
                let numerator = self.builder.ins().fsub(x_vals[i], edge0_vals[i]);
                let denominator = self.builder.ins().fsub(edge1_vals[i], edge0_vals[i]);
                let t_raw = self.builder.ins().fdiv(numerator, denominator);

                // t = clamp(t, 0, 1)
                let t_max = self.builder.ins().fmax(t_raw, zero);
                let t_clamped = self.builder.ins().fmin(t_max, one);

                // result = t * t * (3 - 2 * t)
                let t_squared = self.builder.ins().fmul(t_clamped, t_clamped);
                let two_t = self.builder.ins().fmul(two, t_clamped);
                let three_minus_two_t = self.builder.ins().fsub(three, two_t);
                let result = self.builder.ins().fmul(t_squared, three_minus_two_t);

                result_vals.push(result);
            }
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// fract(x) = x - floor(x) (fractional part)
    fn builtin_fract(
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
    fn builtin_mod(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
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
    fn builtin_sign(
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

    // Matrix Functions (builtinfunctions.adoc:1538-1687)

    /// Component-wise matrix multiply: result[i][j] = x[i][j] * y[i][j]
    fn builtin_matrixCompMult(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (x_vals, x_ty) = &args[0];
        let (y_vals, y_ty) = &args[1];

        if x_ty != y_ty || !x_ty.is_matrix() {
            return Err(GlslError::new(ErrorCode::E0104, "matrixCompMult() requires two matrices of the same type"));
        }

        let mut result_vals = Vec::new();
        for (x_val, y_val) in x_vals.iter().zip(y_vals.iter()) {
            result_vals.push(self.builder.ins().fmul(*x_val, *y_val));
        }

        Ok((result_vals, x_ty.clone()))
    }

    /// Outer product: vec1 × vec2 → matrix
    /// For vec3 × vec3, returns mat3 where result[i][j] = vec1[i] * vec2[j]
    fn builtin_outerProduct(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (vec1_vals, vec1_ty) = &args[0];
        let (vec2_vals, vec2_ty) = &args[1];

        if !vec1_ty.is_vector() || !vec2_ty.is_vector() {
            return Err(GlslError::new(ErrorCode::E0104, "outerProduct() requires two vectors"));
        }

        let vec1_size = vec1_vals.len();
        let vec2_size = vec2_vals.len();

        // Determine result matrix type based on vector sizes
        let result_ty = match (vec1_size, vec2_size) {
            (2, 2) => Type::Mat2,
            (3, 3) => Type::Mat3,
            (4, 4) => Type::Mat4,
            _ => return Err(GlslError::new(ErrorCode::E0104, 
                format!("outerProduct() requires matching vector sizes (got {} and {})", vec1_size, vec2_size))),
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
    fn builtin_transpose(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (m_vals, m_ty) = &args[0];

        if !m_ty.is_matrix() {
            return Err(GlslError::new(ErrorCode::E0104, "transpose() requires a matrix"));
        }

        let (rows, cols) = m_ty.matrix_dims().unwrap();

        // Transpose: result[row][col] = m[col][row]
        // Input is column-major: m_vals[col * rows + row] = m[row][col]
        // Output should be column-major: result_vals[col * rows + row] = result[row][col] = m[col][row]
        // m[col][row] = m_vals[row * cols + col] (since m is stored column-major)
        // So: result_vals[result_col * rows + result_row] = m_vals[result_row * cols + result_col]
        let mut result_vals = Vec::new();
        for result_col in 0..rows {  // Transposed matrix has rows columns
            for result_row in 0..cols {  // Transposed matrix has cols rows
                // result[result_row][result_col] = m[result_col][result_row]
                // m[result_col][result_row] = m_vals[result_row * cols + result_col]
                let old_idx = result_row * cols + result_col;
                result_vals.push(m_vals[old_idx]);
            }
        }

        // Result type is the same (square matrices)
        Ok((result_vals, m_ty.clone()))
    }

    /// Compute matrix determinant
    fn builtin_determinant(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (m_vals, m_ty) = &args[0];

        if !m_ty.is_matrix() {
            return Err(GlslError::new(ErrorCode::E0104, "determinant() requires a matrix"));
        }

        let (rows, cols) = m_ty.matrix_dims().unwrap();
        if rows != cols {
            return Err(GlslError::new(ErrorCode::E0104, "determinant() only supported for square matrices"));
        }

        // Helper to get element at (row, col) from column-major storage
        let get = |row: usize, col: usize| -> Value {
            m_vals[col * rows + row]
        };

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
                return Err(GlslError::new(ErrorCode::E0400, "4x4 determinant not yet implemented"));
            }
            _ => return Err(GlslError::new(ErrorCode::E0104, format!("determinant() not supported for {}-dimensional matrices", rows))),
        };

        Ok((vec![det], Type::Float))
    }

    /// Compute matrix inverse
    fn builtin_inverse(&mut self, args: Vec<(Vec<Value>, Type)>) -> Result<(Vec<Value>, Type), GlslError> {
        let (m_vals, m_ty) = &args[0];

        if !m_ty.is_matrix() {
            return Err(GlslError::new(ErrorCode::E0104, "inverse() requires a matrix"));
        }

        let (rows, cols) = m_ty.matrix_dims().unwrap();
        if rows != cols {
            return Err(GlslError::new(ErrorCode::E0104, "inverse() only supported for square matrices"));
        }

        // Helper to get element at (row, col) from column-major storage
        let get = |row: usize, col: usize| -> Value {
            m_vals[col * rows + row]
        };

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
                result_vals.push(self.builder.ins().fmul(d, inv_det));  // result[0][0]
                result_vals.push(self.builder.ins().fmul(minus_c, inv_det));  // result[1][0]
                result_vals.push(self.builder.ins().fmul(minus_b, inv_det));  // result[0][1]
                result_vals.push(self.builder.ins().fmul(a, inv_det));  // result[1][1]

                Ok((result_vals, m_ty.clone()))
            }
            3 => {
                // For 3x3, use adjugate/determinant method
                // This is complex, so for now return error
                return Err(GlslError::new(ErrorCode::E0400, "3x3 matrix inverse not yet implemented"));
            }
            4 => {
                return Err(GlslError::new(ErrorCode::E0400, "4x4 matrix inverse not yet implemented"));
            }
            _ => return Err(GlslError::new(ErrorCode::E0104, format!("inverse() not supported for {}-dimensional matrices", rows))),
        }
    }
}
