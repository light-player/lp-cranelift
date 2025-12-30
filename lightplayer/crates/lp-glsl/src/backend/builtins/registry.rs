//! Builtin function registry implementation.
//!
//! Provides enum-based registry for builtin functions with support for both
//! JIT (function pointer) and emulator (ELF symbol) linking.

use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::{AbiParam, Signature, types};
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Linkage, Module};

/// Enum identifying builtin functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinId {
    Fixed32Div,
    Fixed32Mul,
    Fixed32Sqrt,
}

impl BuiltinId {
    /// Get the symbol name for this builtin function.
    pub fn name(&self) -> &'static str {
        match self {
            BuiltinId::Fixed32Div => "__lp_fixed32_div",
            BuiltinId::Fixed32Mul => "__lp_fixed32_mul",
            BuiltinId::Fixed32Sqrt => "__lp_fixed32_sqrt",
        }
    }

    /// Get the Cranelift signature for this builtin function.
    pub fn signature(&self) -> Signature {
        let mut sig = Signature::new(CallConv::SystemV);
        match self {
            BuiltinId::Fixed32Div | BuiltinId::Fixed32Mul => {
                // (i32, i32) -> i32
                sig.params.push(AbiParam::new(types::I32));
                sig.params.push(AbiParam::new(types::I32));
                sig.returns.push(AbiParam::new(types::I32));
            }
            BuiltinId::Fixed32Sqrt => {
                // (i32) -> i32
                sig.params.push(AbiParam::new(types::I32));
                sig.returns.push(AbiParam::new(types::I32));
            }
        }
        sig
    }

    /// Get all builtin IDs.
    pub fn all() -> &'static [BuiltinId] {
        &[
            BuiltinId::Fixed32Div,
            BuiltinId::Fixed32Mul,
            BuiltinId::Fixed32Sqrt,
        ]
    }
}

/// Get function pointer for a builtin (JIT mode only).
///
/// Returns the function pointer that can be registered with JITModule.
pub fn get_function_pointer(builtin: BuiltinId) -> *const u8 {
    use lp_builtins::fixed32;
    match builtin {
        BuiltinId::Fixed32Div => fixed32::__lp_fixed32_div as *const u8,
        BuiltinId::Fixed32Mul => fixed32::__lp_fixed32_mul as *const u8,
        BuiltinId::Fixed32Sqrt => fixed32::__lp_fixed32_sqrt as *const u8,
    }
}

/// Declare builtin functions as external symbols.
///
/// This is the same for both JIT and emulator - they both use Linkage::Import.
/// The difference is only in how they're linked:
/// - JIT: Function pointers are registered via symbol_lookup_fn during module creation
/// - Emulator: Symbols are resolved by the linker when linking the static library
pub fn declare_builtins<M: Module>(module: &mut M) -> Result<(), GlslError> {
    for builtin in BuiltinId::all() {
        let name = builtin.name();
        let sig = builtin.signature();

        module
            .declare_function(name, Linkage::Import, &sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Failed to declare builtin '{}': {}", name, e),
                )
            })?;
    }

    Ok(())
}

/// Declare and link builtin functions for JIT mode.
///
/// This declares all builtins as external functions. The function pointers
/// are registered via a symbol lookup function that's added during module creation.
pub fn declare_for_jit<M: Module>(module: &mut M) -> Result<(), GlslError> {
    declare_builtins(module)
}

/// Declare builtin functions as external symbols for emulator mode.
///
/// This declares all builtins as external symbols (Linkage::Import) that will
/// be resolved by the linker when linking the static library.
pub fn declare_for_emulator<M: Module>(module: &mut M) -> Result<(), GlslError> {
    declare_builtins(module)
}
