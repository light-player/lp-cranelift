//! Fixed32 transform implementation for backend2

use crate::backend::transform::fixed32::{convert_signature, rewrite_function};
use crate::backend2::transform::pipeline::{Transform, TransformContext};
use crate::error::GlslError;
use cranelift_codegen::ir::{Function, Signature};

// Use the public re-export
use crate::backend::transform::fixed32::FixedPointFormat;

/// Fixed32 transform - converts F32 to fixed-point representation
pub struct Fixed32Transform {
    format: FixedPointFormat,
}

impl Fixed32Transform {
    /// Create a new Fixed32 transform with the specified format
    pub fn new(format: FixedPointFormat) -> Self {
        Self { format }
    }

    /// Create a Fixed32 transform with default format (Fixed16x16)
    pub fn default() -> Self {
        Self::new(FixedPointFormat::Fixed16x16)
    }
}

impl Transform for Fixed32Transform {
    fn transform_signature(&self, sig: &Signature) -> Signature {
        convert_signature(sig, self.format)
    }

    fn transform_function<M: cranelift_module::Module>(
        &self,
        func: &Function,
        _ctx: &mut TransformContext<'_, M>,
    ) -> Result<Function, GlslError> {
        // Use existing rewrite_function. It will import functions from the new module
        // which are already declared via apply_transform. FuncRef mapping will be handled
        // by rewrite_function's internal ext_func_map mechanism.
        rewrite_function(func, self.format)
    }
}
