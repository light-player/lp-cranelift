//! Host function registry implementation.
//!
//! Provides enum-based registry for host functions with support for JIT linking.

use crate::error::{ErrorCode, GlslError};
use cranelift_codegen::ir::Signature;
use cranelift_codegen::isa::CallConv;
use cranelift_module::{Linkage, Module};

/// Enum identifying host functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostId {
    Debug,
    Println,
}

impl HostId {
    /// Get the symbol name for this host function.
    pub fn name(&self) -> &'static str {
        match self {
            HostId::Debug => "__host_debug",
            HostId::Println => "__host_println",
        }
    }

    /// Get the Cranelift signature for this host function.
    ///
    /// Note: `fmt::Arguments` cannot be passed via C ABI, so we use a simplified
    /// signature. In practice, these functions are linked at link time, not called via FFI.
    /// The signature here is a placeholder - the actual calling convention depends on
    /// how the macros expand and link.
    pub fn signature(&self) -> Signature {
        // For now, we'll use a void signature since fmt::Arguments can't be represented
        // in Cranelift types. The actual implementation handles fmt::Arguments internally.
        // This is a limitation we'll need to work around - host functions may need
        // to be called differently than regular builtins.
        // No parameters (fmt::Arguments is handled internally by macros)
        // No return value
        Signature::new(CallConv::SystemV)
    }

    /// Get all host IDs.
    pub fn all() -> &'static [HostId] {
        &[HostId::Debug, HostId::Println]
    }
}

/// Get function pointer for a host function (JIT mode only).
///
/// Returns the function pointer that can be registered with JITModule.
#[cfg(feature = "std")]
pub fn get_host_function_pointer(host: HostId) -> *const u8 {
    use crate::backend::host::impls;
    match host {
        HostId::Debug => impls::__host_debug as *const u8,
        HostId::Println => impls::__host_println as *const u8,
    }
}

/// Get function pointer for a host function (no_std mode).
///
/// Returns None since host functions require std.
#[cfg(not(feature = "std"))]
pub fn get_host_function_pointer(_host: HostId) -> *const u8 {
    // Return a null pointer - host functions don't work in no_std
    core::ptr::null()
}

/// Declare host functions as external symbols.
///
/// Note: This is currently a placeholder. Host functions use fmt::Arguments
/// which cannot be represented in Cranelift signatures. We may need a different
/// approach for calling these functions from compiled code.
pub fn declare_host_functions<M: Module>(module: &mut M) -> Result<(), GlslError> {
    for host in HostId::all() {
        let name = host.name();
        let sig = host.signature();

        module
            .declare_function(name, Linkage::Import, &sig)
            .map_err(|e| {
                GlslError::new(
                    ErrorCode::E0400,
                    format!("Failed to declare host function '{}': {}", name, e),
                )
            })?;
    }

    Ok(())
}

