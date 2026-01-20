use super::error::ValidationError;
use super::supported;
use crate::CodegenResult;
use crate::ir::{Function, Type};
use alloc::string::ToString;

/// Validate all types used in a function
pub fn validate_types(func: &Function) -> CodegenResult<()> {
    // Validate function signature types
    for &param in func.signature.params.iter() {
        validate_type(param.value_type, "function parameter")?;
    }
    for &ret in func.signature.returns.iter() {
        validate_type(ret.value_type, "function return")?;
    }

    // Validate all block parameters
    for block in func.layout.blocks() {
        for &param in func.dfg.block_params(block) {
            let ty = func.dfg.value_type(param);
            validate_type(ty, "block parameter")?;
        }
    }

    // Validate all value types (this will be called from validate_function,
    // but types are also checked during instruction validation)
    for value in func.dfg.values() {
        let ty = func.dfg.value_type(value);
        validate_type(ty, "value")?;
    }

    Ok(())
}

/// Validate a single type
pub fn validate_type(ty: Type, context: &str) -> CodegenResult<()> {
    // First check if the type is supported at all
    if !supported::is_type_supported(ty) {
        return Err(ValidationError::UnsupportedType {
            ty,
            context: context.to_string(),
        }
        .into());
    }

    // Check if the type requires extensions that aren't always available
    // Note: We don't check extension availability here - that's done in instruction validation
    // This function just checks if the type is theoretically supported
    Ok(())
}
