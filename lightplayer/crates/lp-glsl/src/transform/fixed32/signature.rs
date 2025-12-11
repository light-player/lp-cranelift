//! Function signature conversion for fixed-point transformation.

use crate::transform::fixed32::types::FixedPointFormat;
use cranelift_codegen::ir::{AbiParam, Signature, types};

/// Convert function signature: F32 params/returns → I32
pub fn convert_signature(old_sig: &Signature, format: FixedPointFormat) -> Signature {
    let target_type = format.cranelift_type();
    let mut new_sig = Signature::new(old_sig.call_conv);

    // Convert parameters
    for param in &old_sig.params {
        let new_type = if param.value_type == types::F32 {
            target_type
        } else {
            param.value_type
        };
        new_sig.params.push(AbiParam::new(new_type));
    }

    // Convert return types
    for ret in &old_sig.returns {
        let new_type = if ret.value_type == types::F32 {
            target_type
        } else {
            ret.value_type
        };
        new_sig.returns.push(AbiParam::new(new_type));
    }

    new_sig
}

