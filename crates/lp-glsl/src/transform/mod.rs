//! Transformation passes for Cranelift IR
//!
//! This module contains various transformation passes that can be applied
//! to Cranelift IR after initial code generation.

pub mod fixed_point;
pub mod fixed_point_math;

pub use fixed_point::{FixedPointFormat, convert_floats_to_fixed};
pub use fixed_point_math::{generate_sin_fixed, generate_cos_fixed, generate_tanh_fixed};

#[cfg(feature = "std")]
use crate::pipeline::{TransformationPass, SemanticResult};
#[cfg(feature = "std")]
use crate::error::GlslError;

/// Fixed-point transformation pass
///
/// Note: Currently, fixed-point transformation is applied at the Cranelift IR level,
/// not at the AST level. This is a placeholder for future AST-level transformation.
/// For now, the transformation is still applied during codegen in JIT.
#[cfg(feature = "std")]
pub struct FixedPointTransformation {
    enabled: bool,
    format: Option<FixedPointFormat>,
}

#[cfg(feature = "std")]
impl FixedPointTransformation {
    pub fn new(format: Option<FixedPointFormat>) -> Self {
        Self {
            enabled: format.is_some(),
            format,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn set_format(&mut self, format: Option<FixedPointFormat>) {
        self.format = format;
        self.enabled = format.is_some();
    }
}

#[cfg(feature = "std")]
impl TransformationPass for FixedPointTransformation {
    fn apply<'a>(&self, result: SemanticResult<'a>) -> Result<SemanticResult<'a>, GlslError> {
        if !self.enabled || self.format.is_none() {
            return Ok(result);
        }

        // Note: Current implementation works on Cranelift IR, not AST
        // To move to pipeline, need to transform at AST level
        // This is a larger refactoring - for now, transformation is applied during codegen
        // This is a placeholder that passes through unchanged
        Ok(result)
    }

    fn is_enabled(&self) -> bool {
        self.enabled && self.format.is_some()
    }

    fn name(&self) -> &str {
        "fixed_point"
    }
}

