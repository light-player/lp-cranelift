//! Code generation for GLSL built-in functions

mod geometric;
mod common;
mod trigonometric;
mod interpolation;
mod matrix;
mod helpers;

use crate::codegen::context::CodegenContext;
use crate::semantic::types::Type;
use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::Value;

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
            // Geometric Functions
            "dot" => self.builtin_dot(args),
            "cross" => self.builtin_cross(args),
            "length" => self.builtin_length(args),
            "normalize" => self.builtin_normalize(args),
            "distance" => self.builtin_distance(args),
            
            // Common Functions
            "min" => self.builtin_min(args),
            "max" => self.builtin_max(args),
            "clamp" => self.builtin_clamp(args),
            "abs" => self.builtin_abs(args),
            "sqrt" => self.builtin_sqrt(args),
            "floor" => self.builtin_floor(args),
            "ceil" => self.builtin_ceil(args),
            "fract" => self.builtin_fract(args),
            "mod" => self.builtin_mod(args),
            "sign" => self.builtin_sign(args),
            "pow" => self.builtin_pow(args),
            
            // Angle and Trigonometry Functions
            "radians" => self.builtin_radians(args),
            "degrees" => self.builtin_degrees(args),
            "sin" => self.builtin_sin(args),
            "cos" => self.builtin_cos(args),
            "tan" => self.builtin_tan(args),
            "asin" => self.builtin_asin(args),
            "acos" => self.builtin_acos(args),
            "atan" => self.builtin_atan(args),
            "sinh" => self.builtin_sinh(args),
            "cosh" => self.builtin_cosh(args),
            "tanh" => self.builtin_tanh(args),
            "asinh" => self.builtin_asinh(args),
            "acosh" => self.builtin_acosh(args),
            "atanh" => self.builtin_atanh(args),
            
            // Interpolation Functions
            "mix" => self.builtin_mix(args),
            "step" => self.builtin_step(args),
            "smoothstep" => self.builtin_smoothstep(args),
            
            // Matrix Functions
            "matrixCompMult" => self.builtin_matrixCompMult(args),
            "outerProduct" => self.builtin_outerProduct(args),
            "transpose" => self.builtin_transpose(args),
            "determinant" => self.builtin_determinant(args),
            "inverse" => self.builtin_inverse(args),
            
            _ => Err(GlslError::new(ErrorCode::E0400, format!("built-in function not implemented: {}", name))),
        }
    }
}



